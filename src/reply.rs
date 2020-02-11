#![allow(non_camel_case_types)]

use crate::{from_cstring, IrcIdent, IrcIdentRef, UserMask, UserString};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use std::os::raw::c_char;

/// A type representing a server response. Used with `add_server_response_listener`. It is
/// not recommended you implement this on your own types.
pub trait ServerReply
where
    Self: Sized,
{
    /// The numeric ID of this response.
    const ID: &'static str;
    #[doc(hidden)]
    unsafe fn create(word: *mut *mut c_char, word_eol: *mut *mut c_char) -> Option<Self>;
}

///// A `ServerResponse` corresponding to `RPL_WELCOME` (`001`).
//pub struct RPL_WELCOME {
//    nick: IrcIdent,
//    user: IrcIdent,
//    host: String,
//}
//
//impl RPL_WELCOME {
//    pub fn get_nick(&self) -> IrcIdentRef {
//        self.nick.as_ref()
//    }
//    pub fn get_user(&self) -> IrcIdentRef {
//        self.user.as_ref()
//    }
//    pub fn get_host(&self) -> &str {
//        &self.host
//    }
//}

macro_rules! rpl {
    ($t:ident[$e:expr] { global($word:ident $word_eol:ident) { $($s:stmt;)* } ($this:ident) $([$desc:expr] $name:ident : $ftype:ty [$rtype:ty] get $getter:block parse $parser:block)* }) => {
        rpl!(@RPL (stringify!($t), stringify!($e)) $t[$e] { global($word $word_eol) { $($s;)* } ($this) $([$desc] $name : $ftype [$rtype] get $getter parse $parser)* });
    };
    ($t:ident[$e:expr] empty) => {
        rpl!($t[$e] { global(_a _b) {} (_c) });
    };
    (@RPL ($te:expr, $ee:expr) $t:ident[$e:expr] { global($word:ident $word_eol:ident) { $($s:stmt;)* } ($this:ident) $([$desc:expr] $name:ident : $ftype:ty [$rtype:ty] get $getter:block parse $parser:block)* }) => {
        #[doc = "A `ServerResponse` corresponding to `"]
        #[doc = $te]
        #[doc = "` (`"]
        #[doc = $ee]
        #[doc = "`)"]
        pub struct $t {
            server: IrcIdent,
            target: IrcIdent,
            $(
            $name : $ftype,
            )*
        }

        impl $t {
            #[doc = "The server that sent the response."]
            pub fn server(&self) -> IrcIdentRef {
                self.server.as_ref()
            }
            #[doc = "The target of the response."]
            pub fn target(&self) -> IrcIdentRef {
                self.target.as_ref()
            }
            $(
            #[doc = $desc]
            pub fn $name(&self) -> $rtype {
                let $this = self;
                $getter
            }
            )*
        }

        impl ServerReply for $t {
            const ID: &'static str = stringify!($e);
            unsafe fn create(
                word: *mut *mut c_char,
                word_eol: *mut *mut c_char,
            ) -> Option<Self> {
                let server = IrcIdent(from_cstring((*word.offset(1)).offset(1)));
                let target = IrcIdent(from_cstring(*word.offset(3)));
                let $word = word.offset(4);
                let $word_eol = word_eol.offset(4);
                $(
                $s
                )*
                $(
                let $name = $parser;
                )*
                Some(Self {
                    server,
                    target,
                    $(
                    $name,
                    )*
                })
            }
        }
    }
}

fn parse_datetime(string: impl Into<String>) -> Result<DateTime<Utc>, String> {
    let string = string.into();
    NaiveDateTime::parse_from_str(&string, "%T %b %e %Y")
        .ok()
        .map(|n| Utc.from_utc_datetime(&n))
        .or_else(|| {
            NaiveDateTime::parse_from_str(&string, "%c")
                .ok()
                .map(|n| Utc.from_utc_datetime(&n))
        })
        .or_else(|| {
            DateTime::parse_from_rfc2822(&string)
                .ok()
                .map(|d| d.with_timezone(&Utc))
        })
        .or_else(|| {
            DateTime::parse_from_rfc3339(&string)
                .ok()
                .map(|d| d.with_timezone(&Utc))
        })
        .ok_or(string)
}

rpl!(RPL_WELCOME[001] {
    global(msg _a) {
        let user_str = from_cstring(*msg.offset(6));
    }
    (this)
    ["The welcomed user."]
    user: UserString [&UserString]
        get { &this.user }
        parse { UserString::new(user_str)? }
});

rpl!(RPL_YOURHOST[002] {
    global(msg _a) {
        let server_str = from_cstring(*msg.offset(3));
        let server_str = server_str[..(server_str.len() - 1)].to_string();
        let version_str = from_cstring(*msg.offset(6));
    }
    (this)
    ["The server name."]
    server_name: String [&str]
        get { &this.server_name }
        parse { server_str.to_string() }
    ["The version string."]
    version: String [&str]
        get { &this.version }
        parse { version_str.to_string() }
});

rpl!(RPL_CREATED[003] {
    global(_a msg) {
        let string = from_cstring(*msg.offset(4));
    }
    (this)
    ["The date the server was created."]
    date: Result<DateTime<Utc>, String> [&Result<DateTime<Utc>, String>]
        get { &this.date }
        parse { parse_datetime(string) }
});

rpl!(RPL_MYINFO[004] {
    global(msg _a) {}
    (this)
    ["The name of the server."]
    server_name: IrcIdent [IrcIdentRef]
        get { this.server_name.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The version of the server."]
    version: String [&str]
        get { &this.version }
        parse { from_cstring(*msg.offset(1)) }
    ["The available user modes."]
    usermodes: String [&str]
        get { &this.usermodes }
        parse { from_cstring(*msg.offset(2)) }
    ["The available channel modes."]
    chanmodes: String [&str]
        get { &this.chanmodes }
        parse { from_cstring(*msg.offset(3)) }
});

rpl!(RPL_BOUNCE[005] {
    global(msg _a) {
        let server_string = from_cstring(*msg.offset(2));
    }
    (this)
    ["The alternative server to use."]
    server_name: String [&str]
        get { &this.server_name }
        parse { server_string[..(server_string.len() - 1)].to_string() }
    ["The port number of the alternative server."]
    port_number: u16 [u16]
        get { this.port_number }
        parse { from_cstring(*msg.offset(4)).parse().ok()? }
});

rpl!(RPL_USERHOST[302] {
    global(_a msg) {
        let string = from_cstring((*msg).offset(1));
        let mut vec = Vec::new();
        for reply in string.split(' ') {
            let away_offset = reply.find('=')?;
            let (is_op, nickname) = if let Some(idx) = reply[..away_offset].find('*') {
                (true, IrcIdent(reply[..idx].to_string()))
            } else {
                (false, IrcIdent(reply[..away_offset].to_string()))
            };
            let away_offset = away_offset + 1;
            let is_away = &reply[away_offset..=away_offset] == "-";
            let hostname = reply[(away_offset + 1)..].to_string();
            vec.push(UserReply { nickname, is_op, is_away, hostname });
        };
    }
    (this)
    ["The users that were replied with."]
    replies: Vec<UserReply> [&[UserReply]]
        get { &this.replies }
        parse { vec }
});

#[derive(Clone, Debug, Eq, PartialEq)]
/// A reply entry to `RPL_USERHOST`.
pub struct UserReply {
    nickname: IrcIdent,
    is_op: bool,
    is_away: bool,
    hostname: String,
}

impl UserReply {
    /// The nickname of the user.
    pub fn nickname(&self) -> IrcIdentRef {
        self.nickname.as_ref()
    }
    /// Whether the user is operator.
    pub fn is_op(&self) -> bool {
        self.is_op
    }
    /// Whether the user is away.
    pub fn is_away(&self) -> bool {
        self.is_away
    }
    /// The hostname of the user.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }
}

rpl!(RPL_ISON[303] {
    global(_a msg) {
        let string = from_cstring((*msg).offset(1));
        let mut vec = Vec::new();
        for nick in string.split(' ') {
            vec.push(IrcIdent(nick.to_string()));
        };
    }
    (this)
    ["The list of nicknames that are online."]
    nicks: Vec<IrcIdent> [&[IrcIdent]]
        get { &this.nicks }
        parse { vec }
});

rpl!(RPL_AWAY[301] {
    global(msg eol) {}
    (this)
    ["The nickname of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The away message."]
    message: String [&str]
        get { &this.message }
        parse { from_cstring((*eol.offset(1)).offset(1)) }
});

rpl!(RPL_UNAWAY[305] empty);

rpl!(RPL_NOWAWAY[306] empty);

rpl!(RPL_WHOISUSER[311] {
    global(msg eol) {
        let nick = from_cstring(*msg);
        let user = from_cstring(*msg.offset(1));
        let host = from_cstring(*msg.offset(2));
    }
    (this)
    ["The user being queried."]
    user: UserString [&UserString]
        get { &this.user }
        parse { UserString::from_parts(&nick, &user, &host)? }
    ["The real name of the user."]
    real_name: String [&str]
        get { &this.real_name }
        parse { from_cstring((*eol.offset(4)).offset(1)) }
});

rpl!(RPL_WHOISSERVER[312] {
    global(msg eol) {}
    (this)
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The server of the user."]
    rpl_server: IrcIdent [IrcIdentRef]
        get { this.rpl_server.as_ref() }
        parse { IrcIdent(from_cstring(*msg.offset(1))) }
    ["The server info message."]
    info: String [&str]
        get { &this.info }
        parse { from_cstring((*eol.offset(2)).offset(1)) }
});

rpl!(RPL_WHOISOPERATOR[313] {
    global(msg _a) {}
    (this)
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_WHOISIDLE[317] {
    global(msg _a) {}
    (this)
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The duration this user has been idle for."]
    idle: Duration [Duration]
        get { this.idle }
        parse { Duration::seconds(from_cstring(*msg.offset(1)).parse().ok()?) }
});

rpl!(RPL_ENDOFWHOIS[318] {
    global(msg _a) {}
    (this)
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_WHOISCHANNELS[319] {
    global(msg eol) {}
    (this)
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The channels the user is in."]
    channels: Vec<ChannelEntry> [&[ChannelEntry]]
        get { &this.channels }
        parse {
            let mut vec = Vec::new();
            let channels = from_cstring((*eol.offset(1)).offset(1));
            for channel in channels.split(' ') {
                let (operator, channel) = if &channel[..1] == "@" { (true, &channel[1..]) } else { (false, channel) };
                let (voice, channel) = if &channel[..1] == "+" { (true, &channel[1..]) } else { (false, channel) };
                let channel = IrcIdent(channel.to_string());
                vec.push(ChannelEntry { channel, operator, voice });
            }
            vec
        }
});

/// An entry for `RPL_WHOISCHANNELS`.
pub struct ChannelEntry {
    channel: IrcIdent,
    operator: bool,
    voice: bool,
}

impl ChannelEntry {
    /// The channel that the user is in.
    pub fn channel(&self) -> IrcIdentRef {
        self.channel.as_ref()
    }
    /// Whether the user is an operator.
    pub fn operator(&self) -> bool {
        self.operator
    }
    /// Whether the user has voice.
    pub fn voice(&self) -> bool {
        self.voice
    }
}

rpl!(RPL_WHOWASUSER[314] {
    global(msg eol) {
        let nick = from_cstring(*msg);
        let user = from_cstring(*msg.offset(1));
        let host = from_cstring(*msg.offset(2));
    }
    (this)
    ["The user being queried."]
    user: UserString [&UserString]
        get { &this.user }
        parse { UserString::from_parts(&nick, &user, &host)? }
    ["The real name of the user."]
    realname: String [&str]
        get { &this.realname }
        parse { from_cstring((*eol.offset(4)).offset(1)) }
});

rpl!(RPL_ENDOFWHOWAS[369] {
    global(msg _a) {}
    (this)
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_LIST[322] {
    global(msg eol) {}
    (this)
    ["The channel being listed."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The number of users visible."]
    visible: u32 [u32]
        get { this.visible }
        parse { from_cstring(*msg.offset(1)).parse().ok()? }
    ["The topic of the channel."]
    topic: String [&str]
        get { &this.topic }
        parse { from_cstring((*eol.offset(2)).offset(1)) }
});

rpl!(RPL_LISTEND[323] empty);

rpl!(RPL_UNIQOPIS[325] {
    global(msg _a) {}
    (this)
    ["The channel being queried."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The nick of the user."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg.offset(1))) }
});

rpl!(RPL_CHANNELMODEIS[324] {
    global(msg eol) {}
    (this)
    ["The channel being queried."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The channel mode string."]
    mode: String [&str]
        get { &this.mode }
        parse { from_cstring(*msg.offset(1)) }
    ["The channel mode parameters."]
    params: Vec<String> [&[String]]
        get { &this.params }
        parse {
            let mut vec = Vec::new();
            let string = from_cstring(*eol.offset(2));
            for param in string.split(' ') {
                vec.push(param.to_string());
            }
            vec
        }
});

rpl!(RPL_NOTOPIC[331] {
    global(msg _a) {}
    (this)
    ["The channel being queried."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_TOPIC[332] {
    global(msg eol) {}
    (this)
    ["The channel being queried."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The channel topic."]
    topic: String [&str]
        get { &this.topic }
        parse { from_cstring((*eol.offset(1)).offset(1)) }
});

rpl!(RPL_INVITING[341] {
    global(msg _a) {}
    (this)
    ["The channel being invited to."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The nick of the user being invited."]
    nick: IrcIdent [IrcIdentRef]
        get { this.nick.as_ref() }
        parse { IrcIdent(from_cstring(*msg.offset(1))) }
});

rpl!(RPL_SUMMONING[342] {
    global(msg _a) {}
    (this)
    ["The user being summoned."]
    user: IrcIdent [IrcIdentRef]
        get { this.user.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_INVITELIST[346] {
    global(msg _a) {}
    (this)
    ["The channel being invited to."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The invite mask being invited."]
    invite_mask: UserMask [&UserMask]
        get { &this.invite_mask }
        parse { UserMask::new(from_cstring(*msg.offset(1)))? }
});

rpl!(RPL_ENDOFINVITELIST[347] {
    global(msg _a) {}
    (this)
    ["The channel being invited to."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_EXCEPTLIST[348] {
    global(msg _a) {}
    (this)
    ["The channel being excepted from."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["The exception mask being excepted."]
    exception_mask: UserMask [&UserMask]
        get { &this.exception_mask }
        parse { UserMask::new(from_cstring(*msg.offset(1)))? }
});

rpl!(RPL_ENDOFEXCEPTLIST[349] {
    global(msg _a) {}
    (this)
    ["The channel being excepted from."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_VERSION[351] {
    global(msg eol) {
        let version_string = from_cstring(*msg);
        let debug_offset = version_string.find('.')?;
    }
    (this)
    ["The version of the server."]
    version: String [&str]
        get { &this.version }
        parse { version_string[..debug_offset].to_string() }
    ["The debug level of the server."]
    debug_level: String [&str]
        get { &this.debug_level }
        parse { version_string[(debug_offset + 1)..].to_string() }
    ["The server name."]
    server_name: String [&str]
        get { &this.server_name }
        parse { from_cstring(*msg.offset(1)) }
    ["The server comment."]
    comment: String [&str]
        get { &this.comment }
        parse { from_cstring((*eol.offset(2)).offset(1)) }
});

rpl!(RPL_WHOREPLY[352] {
    global(msg eol) {
        let username = from_cstring(*msg.offset(1));
        let host = from_cstring(*msg.offset(2));
        let nick = from_cstring(*msg.offset(4));
    }
    (this)
    ["The channel the user is in."]
    channel: Option<IrcIdent> [Option<IrcIdentRef>]
        get { this.channel.as_ref().map(IrcIdent::as_ref) }
        parse {
            let string = from_cstring(*msg);
            if &string == "*" { None } else { Some(IrcIdent(string)) }
        }
    ["The user being described."]
    user: UserString [&UserString]
        get { &this.user }
        parse { UserString::from_parts(&nick, &username, &host)? }
    ["The server the user is on."]
    target_server: String [&str]
        get { &this.target_server }
        parse { from_cstring(*msg.offset(3)) }
    ["The flags of the user. Can include `H`, `G`, `*`, `@`, `+`, `&`"]
    flags: String [&str]
        get { &this.flags }
        parse { from_cstring(*msg.offset(5)) }
    ["The hopcount of the user."]
    hopcount: u32 [u32]
        get { this.hopcount }
        parse { from_cstring((*msg.offset(6)).offset(1)).parse().ok()? }
    ["The real name of the user."]
    realname: String [&str]
        get { &this.realname }
        parse { from_cstring(*eol.offset(7)) }
});

rpl!(RPL_ENDOFWHO[315] {
    global(msg _a) {}
    (this)
    ["The originally sent query."]
    query: String [&str]
        get { &this.query }
        parse { from_cstring(*msg) }
});

/// The visibility of an IRC channel.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ChannelVisibility {
    /// A public channel.
    Public,
    /// A secret channel.
    Secret,
    /// A private channel.
    Private,
}

/// A user response that could be just a nickname and could also be a full userstring.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum UserResponse {
    /// A userstring response.
    Full(UserString),
    /// A nickname response.
    Basic(IrcIdent),
}

/// A user entry in `RPL_NAMREPLY`.
pub struct NamreplyUser {
    user: UserResponse,
    role: Option<String>,
}

impl NamreplyUser {
    /// The user that was provided.
    pub fn user(&self) -> &UserResponse {
        &self.user
    }
    /// The user's mode (could be `@`, `+`, `&@`, etc.)
    pub fn role(&self) -> Option<&str> {
        self.role.as_ref().map(String::as_str)
    }
}

rpl!(RPL_NAMREPLY[353] {
    global(msg eol) {}
    (this)
    ["The visibility of the channel."]
    visibility: ChannelVisibility [ChannelVisibility]
        get { this.visibility }
        parse {
            match **msg as u8 {
                b'*' => ChannelVisibility::Private,
                b'@' => ChannelVisibility::Secret,
                _ => ChannelVisibility::Public,
            }
        }
    ["The channel the user is in, or `None` if global."]
    channel: Option<IrcIdent> [Option<IrcIdentRef>]
        get { this.channel.as_ref().map(IrcIdent::as_ref) }
        parse {
            let channel = from_cstring(*msg.offset(1));
            if &channel == "*" { None } else { Some(IrcIdent(channel)) }
        }
    ["A list of all the users in the response, coupled with their channel roles."]
    users: Vec<NamreplyUser> [&[NamreplyUser]]
        get { &this.users }
        parse {
            let mut vec = Vec::new();
            let string = from_cstring((*eol.offset(2)).offset(1));
            for user in string.split(' ') {
                let mut role = None;
                let user_str;
                if b"&@+".contains(&user.as_bytes()[0]){
                    user_str = user;
                } else if b"&@+".contains(&user.as_bytes()[1]) {
                    role = Some(user[..1].to_string());
                    user_str = &user[1..];
                } else {
                    role = Some(user[..2].to_string());
                    user_str = &user[2..];
                }
                let user = UserString::new(user_str)
                        .map_or_else(|| UserResponse::Basic(IrcIdent(user_str.to_string())),
                            UserResponse::Full);
                vec.push(NamreplyUser { user, role });
            }
            vec
        }
});

rpl!(RPL_ENDOFNAMES[366] {
    global(msg _a) {}
    (this)
    ["The original query."]
    query: String [&str]
        get { &this.query }
        parse { from_cstring(*msg) }
});

//todo RPL_LINKS/ENDOFLINKS

rpl!(RPL_BANLIST[367] {
    global(msg _a) {}
    (this)
    ["The channel being queried."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
    ["A banmask in the channel."]
    mask: UserMask [&UserMask]
        get { &this.mask }
        parse { UserMask::new(from_cstring(*msg.offset(1)))? }
});

rpl!(RPL_ENDOFBANLIST[368] {
    global(msg _a) {}
    (this)
    ["The channel being queried."]
    channel: IrcIdent [IrcIdentRef]
        get { this.channel.as_ref() }
        parse { IrcIdent(from_cstring(*msg)) }
});

rpl!(RPL_INFO[371] {
    global(_a eol) {}
    (this)
    ["The info string."]
    info: String [&str]
        get { &this.info }
        parse { from_cstring((*eol).offset(1)) }
});

rpl!(RPL_ENDOFINFO[374] empty);

rpl!(RPL_MOTDSTART[375] {
    global(msg _a) {}
    (this)
    ["The server issuing the MOTD."]
    iss_server: String [&str]
        get { &this.iss_server }
        parse { from_cstring(*msg.offset(1)) }
});

rpl!(RPL_MOTD[372] {
    global(_a eol) {}
    (this)
    ["The MOTD line."]
    text: String [&str]
        get { &this.text }
        parse { from_cstring(*eol.offset(1)) }
});

rpl!(RPL_ENDOFMOTD[376] empty);

rpl!(RPL_YOUREOPER[381] empty);

rpl!(RPL_REHASHING[382] {
    global(msg _a) {}
    (this)
    ["The config file being reread."]
    config_file: String [&str]
        get { &this.config_file }
        parse { from_cstring(*msg) }
});

rpl!(RPL_YOURESERVICE[383] {
    global(msg _a) {}
    (this)
    ["The service that you now are."]
    service: IrcIdent [IrcIdentRef]
        get { this.service.as_ref() }
        parse { IrcIdent(from_cstring(*msg.offset(3))) }
});

rpl!(RPL_TIME[391] {
    global(msg eol) {}
    (this)
    ["The server whose time it is."]
    rpl_server: String [&str]
        get { &this.rpl_server }
        parse { from_cstring(*msg) }
    ["The time of the server."]
    time: Result<DateTime<Utc>, String> [&Result<DateTime<Utc>, String>]
        get { &this.time }
        parse { parse_datetime(from_cstring((*eol.offset(1)).offset(1))) }
});

rpl!(USERSSTART[392] empty);

rpl!(RPL_USERS[393] {
    global(msg _a) {}
    (this)
    ["The username of the user."]
    username: String [&str]
        get { &this.username }
        parse { from_cstring((*msg).offset(1)) }
    ["The terminal of the user."]
    ttyline: String [&str]
        get { &this.ttyline }
        parse { from_cstring(*msg.offset(1)) }
    ["The host of the user."]
    host: String [&str]
        get { &this.host }
        parse { from_cstring(*msg.offset(22)) }
});

rpl!(ENDOFUSERS[394] empty);

rpl!(NOUSERS[395] empty);

// todo /TRACE

rpl!(RPL_STATSLINKINFO[211] {
    global(msg _a) {}
    (this)
    ["The connection identifier."]
    linkname: String [&str]
        get { &this.linkname }
        parse { from_cstring(*msg) }
    ["The number of sent messages."]
    sent_messages: u64 [u64]
        get { this.sent_messages }
        parse { from_cstring(*msg.offset(2)).parse().ok()? }
    ["The number of sent kilobytes."]
    sent_kb: u64 [u64]
        get { this.sent_kb }
        parse { from_cstring(*msg.offset(3)).parse().ok()? }
    ["The number of received messages."]
    received_messages: u64 [u64]
        get { this.received_messages }
        parse { from_cstring(*msg.offset(4)).parse().ok()? }
    ["The number of received kilobytes."]
    received_kb: u64 [u64]
        get { this.received_kb }
        parse { from_cstring(*msg.offset(5)).parse().ok()? }
    ["The uptime of this server."]
    uptime: Duration [Duration]
        get { this.uptime }
        parse { Duration::seconds(from_cstring(*msg.offset(6)).parse().ok()?) }
});

rpl!(RPL_STATSCOMMANDS[212] {
    global(msg _a) {}
    (this)
    ["The command being reported."]
    command: String [&str]
        get { &this.command }
        parse { from_cstring(*msg) }
    ["The number of times the command was run."]
    runs: u64 [u64]
        get { this.runs }
        parse { from_cstring(*msg.offset(1)).parse().ok()? }
    ["The number of bytes processed via this command."]
    bytes: u64 [u64]
        get { this.bytes }
        parse { from_cstring(*msg.offset(2)).parse().ok()? }
    ["The remote count."]
    remotes: u64 [u64]
        get { this.remotes }
        parse { from_cstring(*msg.offset(2)).parse().ok()? }
});

rpl!(RPL_ENDOFSTATS[219] {
    global(msg _a) {}
    (this)
    ["The original stats query."]
    query: String [&str]
        get { &this.query }
        parse { from_cstring(*msg) }
});

rpl!(RPL_STATSUPTIME[242] {
    global(msg _a) {}
    (this)
    ["The uptime of the server."]
    uptime: Duration [Duration]
        get { this.uptime }
        parse {
            let days = from_cstring(*msg.offset(2)).parse().ok()?;
            let time_string = from_cstring(*msg.offset(4));
            let (hours, rest) = time_string.split_at(time_string.find(':')?);
            let (minutes, seconds) = rest.split_at(rest.find(':')?);
            Duration::days(days) + Duration::hours(hours.parse().ok()?) +
                Duration::minutes(minutes.parse().ok()?) + Duration::seconds(seconds.parse().ok()?)
        }
});

//todo RPL_STATSOLINE

rpl!(RPL_UMODEIS[221] {
    global(_a eol) {}
    (this)
    ["The user mode string."]
    mode: String [&str]
        get { &this.mode }
        parse { from_cstring(*eol) }
});

//todo RPL_SERVLIST/END

rpl!(RPL_LUSERCLIENT[251] {
    global(msg _a) {}
    (this)
    ["The number of users."]
    users: u64 [u64]
        get { this.users }
        parse { from_cstring(*msg.offset(2)).parse().ok()? }
    ["The number of services."]
    services: u64 [u64]
        get { this.services }
        parse { from_cstring(*msg.offset(5)).parse().ok()? }
    ["The number of servers."]
    servers: u64 [u64]
        get { this.servers }
        parse { from_cstring(*msg.offset(8)).parse().ok()? }
});

rpl!(RPL_LUSEROP[252] {
    global(msg _a) {}
    (this)
    ["The number of operators online."]
    operators: u64 [u64]
        get { this.operators }
        parse { from_cstring(*msg).parse().ok()? }
});

rpl!(RPL_LUSERUNKNOWN[253] {
    global(msg _a) {}
    (this)
    ["The number of unknown connections."]
    unknown: u64 [u64]
        get { this.unknown }
        parse { from_cstring(*msg).parse().ok()? }
});

rpl!(RPL_LUSERCHANNELS[254] {
    global(msg _a) {}
    (this)
    ["The number of channels."]
    channels: u64 [u64]
        get { this.channels }
        parse { from_cstring(*msg).parse().ok()? }
});

rpl!(RPL_LUSERME[255] {
    global(msg _a) {}
    (this)
    ["The number of users online."]
    users: u64 [u64]
        get { this.users }
        parse { from_cstring(*msg.offset(2)).parse().ok()? }
    ["The number of channels."]
    channels: u64 [u64]
        get { this.channels }
        parse { from_cstring(*msg.offset(5)).parse().ok()? }
});

rpl!(RPL_ADMINME[256] {
    global(msg _a) {}
    (this)
    ["The server name."]
    rpl_server: String [&str]
        get { &this.rpl_server }
        parse { from_cstring(*msg) }
});

rpl!(RPL_ADMINLOC1[257] {
    global(_a eol) {}
    (this)
    ["The info string (usually city, state, and country)."]
    info: String [&str]
        get { &this.info }
        parse { from_cstring((*eol).offset(1)) }
});

rpl!(RPL_ADMINLOC2[258] {
    global(_a eol) {}
    (this)
    ["The info string (usually institution name)."]
    info: String [&str]
        get { &this.info }
        parse { from_cstring((*eol).offset(1)) }
});

rpl!(RPL_ADMINEMAIL[259] {
    global(_a eol) {}
    (this)
    ["The admin email address."]
    email: String [&str]
        get { &this.email }
        parse { from_cstring((*eol).offset(1)) }
});

rpl!(RPL_TRYAGAIN[263] {
    global(msg _a) {}
    (this)
    ["The command you should wait before trying again."]
    command: String [&str]
        get { &this.command }
        parse { from_cstring(*msg) }
});

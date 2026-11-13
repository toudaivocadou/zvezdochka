use std::{
    borrow::Cow,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::Error;
use camino::{Utf8Path, Utf8PathBuf};
use poise::serenity_prelude::{
    Attachment, ButtonStyle, CreateButton, ReactionType, small_fixed_array::FixedString,
};

use crate::bot::{Data, TicketId};

pub const FILE_SIZE_LIMIT_BYTES: u32 = 5242880; // 5 mb

pub const EMOJI_GOOD: &'static str = "🟢";
pub const EMOJI_YABAI: &'static str = "⛔";
pub const EMOJI_CHECK: &'static str = "👁️";

pub const EMOJI_CONFIRM: &'static str = "✅";
pub const EMOJI_CANCEL: &'static str = "❌";

pub const EMOJI_LIST_ADD: &'static str = "➕";
pub const EMOJI_LIST_REMOVE: &'static str = "🗑️";

pub const EMOJI_REQUIRED: &'static str = "‼️";
pub const EMOJI_RECOMMENDED: &'static str = "❗";
pub const EMOJI_OPTIONAL: &'static str = "ℹ️";

pub const EMOJI_FLIP_TO_TRACK: &'static str = "💽";
pub const EMOJI_FLIP_TO_INFO: &'static str = "📒";
pub const EMOJI_FLIP_TO_SNSLINK: &'static str = "🔗";
pub const EMOJI_FLIP_TO_ILLUSTRATIONS: &'static str = "🖼️";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathSection {
    Album,
    News,
    Member,
    Work,
    Image,
}

pub fn verify_and_canonicalise_path(
    base_dir: &Utf8Path,
    path_prefix: PathSection,
    input_path: &str,
) -> Result<PathBuf, Error> {
    let path_prefix = match path_prefix {
        PathSection::Album => Path::new("albums"),
        PathSection::News => Path::new("news"),
        PathSection::Member => Path::new("members"),
        PathSection::Work => Path::new("works"),
        PathSection::Image => Path::new("image"),
    };
    let input_path = Path::new(input_path);
    let base_path = base_dir.as_std_path();

    let collected_paths = [base_path, path_prefix, input_path]
        .into_iter()
        .collect::<PathBuf>();

    let canonicalised_path = collected_paths.canonicalize()?;
    match canonicalised_path.starts_with(base_path) {
        true => return Ok(canonicalised_path),
        false => return Err(Error::msg("このパスは作業の範囲を離脱しました。")),
    }
}

pub fn check_attachmemt_is_picture_and_under_flimit(attachment: &Attachment) -> Result<(), Error> {
    if let None = attachment.dimensions() {
        return Err(Error::msg("Attachment is not an image!"));
    }

    let path = Path::new(&attachment.filename);
    match path.extension() {
        Some(ext) => {
            let lossy = match ext.to_str() {
                Some(s) => s,
                None => return Err(Error::msg("ファイルの拡張子に不法文字があります")),
            };

            if lossy != "png"
                || lossy != "jpg"
                || lossy != "jpeg"
                || lossy != "avif"
                || lossy != "webp"
                || lossy != "gif"
            {
                return Err(Error::msg(format!(
                    "ファイルタイプが不明です - .{}は認知できません。",
                    lossy
                )));
            }
        }
        None => return Err(Error::msg("ファイルの拡張子が不明です")),
    }

    if attachment.size > FILE_SIZE_LIMIT_BYTES {
        return Err(Error::msg(format!(
            "このファイルは最大サイズを超過しました: {FILE_SIZE_LIMIT_BYTES} < {}",
            attachment.size
        )));
    }

    return Ok(());
}

pub fn check_file_exists(
    base_dir: &Utf8Path,
    path_prefix: PathSection,
    input_path: &str,
) -> Result<(), Error> {
    let canonical_path = verify_and_canonicalise_path(base_dir, path_prefix, input_path)?;

    std::fs::File::open(canonical_path)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentInteractionTypes {
    Ticket(TicketComponents),
    Singular(SingularEditComponents),
    Multiple(MultipleEditComponents),
    PageTurners(PageTurners),
    EditConfirmCancel(EditConfirmCancel),
}

impl ComponentInteractionTypes {
    pub fn to_component_id_cow(&self) -> Cow<'static, str> {
        let s = self.to_string();
        Cow::Owned(s)
    }

    pub fn to_button_label(&self) -> Cow<'static, str> {
        match self {
            ComponentInteractionTypes::Ticket(ticket_components) => match ticket_components {
                TicketComponents::TicketOpen => Cow::Borrowed("新しい作業を始める"),
                TicketComponents::TicketClose => Cow::Borrowed("この作業をキャンセルする"),
                TicketComponents::TicketSubmit => Cow::Borrowed("この作業を投稿する"),
            },
            ComponentInteractionTypes::Singular(singular_edit_components) => {
                match singular_edit_components {
                    SingularEditComponents::BasicInfo => Cow::Borrowed("基本情報"),
                    SingularEditComponents::Authors => Cow::Borrowed("作成者・投稿者"),
                    SingularEditComponents::Content => Cow::Borrowed("ページ内容・曲の歌詞"),
                    SingularEditComponents::Link => Cow::Borrowed("主のリンク"),
                    SingularEditComponents::Thumbnail => Cow::Borrowed("サムネイル"),
                    SingularEditComponents::ArtistInfo => Cow::Borrowed("メンバー基本情報"),
                    SingularEditComponents::AdditionlArtistInfo => {
                        Cow::Borrowed("メンバー選択情報")
                    }
                    SingularEditComponents::Duration => Cow::Borrowed("曲の長さ"),
                }
            }
            ComponentInteractionTypes::Multiple(multiple_edit_components) => {
                let part_a = match multiple_edit_components.interact_type {
                    MultiInteractType::SnsLink => "SNSリンク",
                    MultiInteractType::Illustrations => "イラスト",
                    MultiInteractType::AlbumTracks => "アルバムトラック",
                };
                match multiple_edit_components.action_type {
                    ListInteraction::Add => Cow::Owned(format!("{part_a}を追加")),
                    ListInteraction::Remove => Cow::Owned(format!("{part_a}を削除")),
                    ListInteraction::Edit(idx) => Cow::Owned(format!("{part_a} #{idx}を編集")),
                }
            }
            ComponentInteractionTypes::PageTurners(page_turners) => match page_turners {
                PageTurners::ToInfo => Cow::Borrowed("情報入力ページへ"),
                PageTurners::ToSnsLinks => Cow::Borrowed("SNSリンク入力ページへ"),
                PageTurners::ToIllustrations => Cow::Borrowed("イラスト入力ページへ"),
                PageTurners::ToAlbumTracks => Cow::Borrowed("アルバムのトラック入力ページへ"),
            },
            ComponentInteractionTypes::EditConfirmCancel(edit_confirm_cancel) => {
                match edit_confirm_cancel {
                    EditConfirmCancel::Cancel => Cow::Borrowed("この編集をキャンセルする"),
                    EditConfirmCancel::Confirm => Cow::Borrowed("この編集を確定する"),
                }
            }
        }
    }

    pub const fn associated_emoji(&self) -> &'static str {
        match self {
            ComponentInteractionTypes::Ticket(ticket_components) => match ticket_components {
                TicketComponents::TicketOpen => "📩",
                TicketComponents::TicketClose => "🔒",
                TicketComponents::TicketSubmit => "📤",
            },
            ComponentInteractionTypes::Singular(singular_edit_components) => {
                match singular_edit_components {
                    SingularEditComponents::BasicInfo => "📋",
                    SingularEditComponents::Authors => "👥",
                    SingularEditComponents::Content => "📖",
                    SingularEditComponents::Link => "🔗",
                    SingularEditComponents::Thumbnail => "🖼️",
                    SingularEditComponents::ArtistInfo => "🪪",
                    SingularEditComponents::AdditionlArtistInfo => "💳",
                    SingularEditComponents::Duration => "⏱️",
                }
            }
            ComponentInteractionTypes::Multiple(multiple_edit_components) => {
                match multiple_edit_components.action_type {
                    ListInteraction::Add => "➕",
                    ListInteraction::Remove => "❌",
                    ListInteraction::Edit(_) => match multiple_edit_components.interact_type {
                        MultiInteractType::SnsLink => "🔗",
                        MultiInteractType::Illustrations => "🖼️",
                        MultiInteractType::AlbumTracks => "💽",
                    },
                }
            }
            ComponentInteractionTypes::PageTurners(page_turners) => match page_turners {
                PageTurners::ToInfo => "📋",
                PageTurners::ToSnsLinks => "🔗",
                PageTurners::ToIllustrations => "🖼️",
                PageTurners::ToAlbumTracks => "💽",
            },
            ComponentInteractionTypes::EditConfirmCancel(edit_confirm_cancel) => {
                match edit_confirm_cancel {
                    EditConfirmCancel::Cancel => "🚫",
                    EditConfirmCancel::Confirm => "✅",
                }
            }
        }
    }

    pub fn default_associated_style(&self) -> Option<ButtonStyle> {
        match self {
            ComponentInteractionTypes::Ticket(ticket_components) => match ticket_components {
                TicketComponents::TicketOpen => None,
                TicketComponents::TicketClose => Some(ButtonStyle::Danger),
                TicketComponents::TicketSubmit => Some(ButtonStyle::Success),
            },
            ComponentInteractionTypes::Singular(_) => None,
            ComponentInteractionTypes::Multiple(mce) => match mce.action_type {
                ListInteraction::Add => None,
                ListInteraction::Remove => Some(ButtonStyle::Danger),
                ListInteraction::Edit(_) => None,
            },
            ComponentInteractionTypes::PageTurners(_) => Some(ButtonStyle::Secondary),
            ComponentInteractionTypes::EditConfirmCancel(edit_confirm_cancel) => {
                match edit_confirm_cancel {
                    EditConfirmCancel::Cancel => Some(ButtonStyle::Danger),
                    EditConfirmCancel::Confirm => Some(ButtonStyle::Success),
                }
            }
        }
    }

    pub fn to_button(self) -> CreateButton<'static> {
        let id = self.to_component_id_cow();
        let label = self.to_button_label();
        let emoji = ReactionType::Unicode(FixedString::from_static_trunc(self.associated_emoji()));

        let button = CreateButton::new(id).label(label).emoji(emoji);
        if let Some(style) = self.default_associated_style() {
            return button.style(style);
        }
        button
    }
}

impl Display for ComponentInteractionTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (interaction_comps, inner) = match self {
            ComponentInteractionTypes::Ticket(ticket_components) => {
                ("Ticket", ticket_components.to_string())
            }
            ComponentInteractionTypes::Singular(singular_edit_components) => {
                ("Singular", singular_edit_components.to_string())
            }
            ComponentInteractionTypes::Multiple(multiple_edit_components) => {
                ("Multiple", multiple_edit_components.to_string())
            }
            ComponentInteractionTypes::PageTurners(page_turners) => {
                ("PageTurner", page_turners.to_string())
            }
            ComponentInteractionTypes::EditConfirmCancel(edit_confirm_cancel) => {
                ("EditConfirmCancel", edit_confirm_cancel.to_string())
            }
        };

        write!(f, "{interaction_comps}_{inner}")
    }
}

impl FromStr for ComponentInteractionTypes {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (front, back) = match s.split_once("_") {
            Some(a) => a,
            None => {
                return Err(Error::msg(format!(
                    "bad component interaction type pattern: {s}"
                )));
            }
        };

        match front {
            "Ticket" => Ok(ComponentInteractionTypes::Ticket(
                TicketComponents::from_str(back)?,
            )),
            "Singular" => Ok(ComponentInteractionTypes::Singular(
                SingularEditComponents::from_str(back)?,
            )),
            "Multiple" => Ok(ComponentInteractionTypes::Multiple(
                MultipleEditComponents::from_str(back)?,
            )),
            "PageTurner" => Ok(ComponentInteractionTypes::PageTurners(
                PageTurners::from_str(back)?,
            )),
            "EditConfirmCancel" => Ok(ComponentInteractionTypes::EditConfirmCancel(
                EditConfirmCancel::from_str(back)?,
            )),
            a => Err(Error::msg(format!(
                "bad component interaction type pattern: {a}"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TicketComponents {
    TicketOpen,
    TicketClose,
    TicketSubmit,
}

impl Display for TicketComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            TicketComponents::TicketOpen => "TicketOpen",
            TicketComponents::TicketClose => "TicketClose",
            TicketComponents::TicketSubmit => "TicketSubmit",
        };

        write!(f, "{m}")
    }
}

impl FromStr for TicketComponents {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TicketOpen" => Ok(TicketComponents::TicketOpen),
            "TicketClose" => Ok(TicketComponents::TicketClose),
            "TicketSubmit" => Ok(TicketComponents::TicketSubmit),
            a => return Err(Error::msg(format!("bad track pattern: {a}"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SingularEditComponents {
    BasicInfo,
    Authors,
    Content,
    Link,
    Thumbnail,
    ArtistInfo,
    AdditionlArtistInfo,
    Duration,
}

impl Display for SingularEditComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            SingularEditComponents::BasicInfo => "BasicInfo",
            SingularEditComponents::Authors => "Authors",
            SingularEditComponents::Content => "Content",
            SingularEditComponents::Link => "Link",
            SingularEditComponents::Thumbnail => "Thumbnail",
            SingularEditComponents::ArtistInfo => "ArtistInfo",
            SingularEditComponents::AdditionlArtistInfo => "AdditionlArtistInfo",
            SingularEditComponents::Duration => "Duration",
        };

        write!(f, "{m}")
    }
}

impl FromStr for SingularEditComponents {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BasicInfo" => Ok(SingularEditComponents::BasicInfo),
            "Authors" => Ok(SingularEditComponents::Authors),
            "Content" => Ok(SingularEditComponents::Content),
            "Link" => Ok(SingularEditComponents::Link),
            "Thumbnail" => Ok(SingularEditComponents::Thumbnail),
            "ArtistInfo" => Ok(SingularEditComponents::ArtistInfo),
            "AdditionlArtistInfo" => Ok(SingularEditComponents::AdditionlArtistInfo),
            "Duration" => Ok(SingularEditComponents::Duration),
            a => return Err(Error::msg(format!("bad singular edit pattern: {a}"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageTurners {
    ToInfo,
    ToSnsLinks,
    ToIllustrations,
    ToAlbumTracks,
}

impl Display for PageTurners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            PageTurners::ToInfo => "ToInfo",
            PageTurners::ToSnsLinks => "ToSnsLinks",
            PageTurners::ToIllustrations => "ToIllustrations",
            PageTurners::ToAlbumTracks => "ToAlbumTracks",
        };

        write!(f, "{m}")
    }
}

impl FromStr for PageTurners {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ToInfo" => Ok(PageTurners::ToInfo),
            "ToSnsLinks" => Ok(PageTurners::ToSnsLinks),
            "ToIllustrations" => Ok(PageTurners::ToIllustrations),
            "ToAlbumTracks" => Ok(PageTurners::ToAlbumTracks),
            a => return Err(Error::msg(format!("bad misc work edit pattern: {a}"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditConfirmCancel {
    Cancel,
    Confirm,
}

impl Display for EditConfirmCancel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            EditConfirmCancel::Cancel => "Cancel",
            EditConfirmCancel::Confirm => "Confirm",
        };

        write!(f, "{m}")
    }
}

impl FromStr for EditConfirmCancel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cancel" => Ok(EditConfirmCancel::Cancel),
            "Confirm" => Ok(EditConfirmCancel::Confirm),
            a => return Err(Error::msg(format!("bad edit confirm cancel pattern: {a}"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MultipleEditComponents {
    pub interact_type: MultiInteractType,
    pub action_type: ListInteraction,
}

impl Display for MultipleEditComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interact = self.interact_type.to_string();
        let action = self.action_type.to_string();
        write!(f, "{interact}_{action}")
    }
}

impl FromStr for MultipleEditComponents {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (front, back) = match s.split_once("_") {
            Some(a) => a,
            None => {
                return Err(Error::msg(format!(
                    "bad multiple edit component pattern: {s}"
                )));
            }
        };

        let interact_type = MultiInteractType::from_str(front)?;
        let action_type = ListInteraction::from_str(back)?;
        Ok(MultipleEditComponents {
            interact_type,
            action_type,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MultiInteractType {
    SnsLink,
    Illustrations,
    AlbumTracks,
}

impl Display for MultiInteractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = match self {
            MultiInteractType::SnsLink => "SNSLINKS",
            MultiInteractType::Illustrations => "ILLUSTRATIONS",
            MultiInteractType::AlbumTracks => "ALBUMTRACKS",
        };
        write!(f, "{a}")
    }
}

impl FromStr for MultiInteractType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SNSLINKS" => Ok(MultiInteractType::SnsLink),
            "ILLUSTRATIONS" => Ok(MultiInteractType::Illustrations),
            "ALBUMTRACKS" => Ok(MultiInteractType::AlbumTracks),
            a => return Err(Error::msg(format!("bad multi interact type pattern: {a}"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListInteraction {
    Add,
    Remove,
    Edit(u8),
}

impl Display for ListInteraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListInteraction::Add => write!(f, "ADD"),
            ListInteraction::Remove => write!(f, "REMOVE"),
            ListInteraction::Edit(u) => write!(f, "{}", u),
        }
    }
}

impl FromStr for ListInteraction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ADD" => Ok(ListInteraction::Add),
            "REMOVE" => Ok(ListInteraction::Remove),
            remainder => {
                let idx = u8::from_str(remainder)?;
                Ok(ListInteraction::Edit(idx))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModalSubmitIds {
    Singular(SingularEditComponents),
    Multiple(MultipleEditComponents),
    Ticket(),
}

pub fn ticket_thread_name(data: Arc<Data>, ticket: TicketId) -> String {
    format!("{}-{}", data.config.site_prefix, ticket.get())
}

pub fn ticket_work_path(data: Arc<Data>, ticket: TicketId) -> Utf8PathBuf {
    let new_path = ticket_thread_name(data, ticket);
    Utf8PathBuf::from_str(&new_path).unwrap() // the err is !, which means this is infalliable
}

const TICKET_OPEN_BUTTON_INTERACTION_ID: &'static str = "ZVEZDOCHKA_OPEN_TICKET";
const TICKET_COMPONENT_PREFIX: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT";
const ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO";
const ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS";
const ZVEZDOCHKA_TICKET_COMPONENT_CONTENT: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_CONTENT";
const ZVEZDOCHKA_TICKET_COMPONENT_LINK: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_LINK";
const ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL";
const ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO";
const ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO";
const ZVEZDOCHKA_TICKET_COMPONENT_DURATION: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_DURATION";

// SNS Links
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_1: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_1";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_2: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_2";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_3: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_3";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_4: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_4";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_5: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_5";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_6: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_6";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_7: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_7";
const ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_8: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_8";
const ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK";
const ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK";

// illustrations
const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_1: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_1";
const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_2: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_2";
const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_3: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_3";
const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_4: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_4";
const ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_5: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_5";
const ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION";
const ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION";
// tracks
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_1: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_1";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_2: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_2";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_3: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_3";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_4: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_4";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_5: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_5";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_6: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_6";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_7: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_7";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_8: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_8";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_9: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_9";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_10: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_10";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_11: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_11";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_12: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_12";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_13: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_13";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_14: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_14";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_15: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_15";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_16: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_16";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_17: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_17";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_18: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_18";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_19: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_19";
const ZVEZDOCHKA_TICKET_COMPONENT_TRACK_20: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_TRACK_20";
const ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK";
const ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK: &'static str = "ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK";
const ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK_INDEXED: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK_INDEXED";
// cancel + confirm
const ZVEZDOCHKA_TICKET_COMPONENT_CANCEL_WORK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_CANCEL_WORK";
const ZVEZDOCHKA_TICKET_COMPONENT_CONFIRM_WORK: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_CONFIRM_WORK";
const ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_INFOPAGE: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_INFOPAGE";
const ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_TRACKSPAGE: &'static str =
    "ZVEZDOCHKA_TICKET_COMPONENT_ALBUM_TRACKSPAGE";

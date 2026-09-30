use std::{borrow::Cow, fmt::Display, gca, sync::Arc, time::Duration};

use ammonia::clean;
use anyhow::Error;
use base64::Engine;
use chrono::{NaiveDate, Utc};
use fancy_duration::FancyDuration;
use poise::{
    Modal,
    serenity_prelude::{
        Attachment, ChannelId, CreateActionRow, CreateButton, CreateComponent, CreateTextDisplay,
        EditMessage, GenericChannelId, Http, MessageId, ReactionType,
        small_fixed_array::{FixedArray, FixedString},
    },
};
use pulldown_cmark::{Event, Options, Parser};
use std::fmt::Debug;
use url::Url;

use crate::{
    bot::{
        Data,
        event::{
            ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO, ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS,
            ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO, ZVEZDOCHKA_TICKET_COMPONENT_CANCEL_WORK,
            ZVEZDOCHKA_TICKET_COMPONENT_CONFIRM_WORK, ZVEZDOCHKA_TICKET_COMPONENT_CONTENT,
            ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION,
            ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK, ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK,
            ZVEZDOCHKA_TICKET_COMPONENT_DURATION, ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO,
            ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_1, ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_2,
            ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_3, ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_4,
            ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_5, ZVEZDOCHKA_TICKET_COMPONENT_LINK,
            ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION, ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK,
            ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK, ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_1,
            ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_2, ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_3,
            ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_4, ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_5,
            ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_6, ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_7,
            ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_8, ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_1, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_2,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_3, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_4,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_5, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_6,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_7, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_8,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_9, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_10,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_11, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_12,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_13, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_14,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_15, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_16,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_17, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_18,
            ZVEZDOCHKA_TICKET_COMPONENT_TRACK_19, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_20,
        },
    },
    site::album::{Illustration, Track},
};

mod album;
mod artist;
mod news;
mod song;

pub const FILE_SIZE_LIMIT_BYTES: u32 = 10485760; // 10 mb
pub const DISCORD_MESSAGE_EXPLATINATION: &'static str = r#"
### 状態絵文字の説明(まだ入力がない場合):
- ‼️: この入力は必要
- ❗: この入力は鑑賞
- ℹ️: この入力は選択
### 状態絵文字の説明(まだ入力がある場合)
- ✅: この入力は良好
- ❌: この入力は不良
"#;

pub enum WorkType {
    Album,
    Artist,
    News,
    Song,
}

pub struct WorkState {
    pub current_work_message: MessageId,
    pub work_state: WorkStateData,
}

pub enum AlbumWorkStatePage {
    BasicInfo,
    Tracks,
}

pub enum WorkStateData {
    Album {
        page: AlbumWorkStatePage,
        basic_info: ModalWrapper<BasicInfo>,
        authors: ModalWrapper<Authors>,
        content: ModalWrapper<PageContent>,
        link: ModalWrapper<MainLink>,
        sns_links: ModalListWrapper<SnsLink, 8>,
        thumbnail: ModalWrapper<Thumbnail>,
        illustrations: ModalListWrapper<TitleAndIllustraion, 5>,
        tracks: ModalListWrapper<AlbumTitleAndTrack, 20>,
    },
    Artist {
        artist_info: ModalWrapper<MemberBasicInfo>,
        additional_artist_info: ModalWrapper<ExtendedMemberBasicInfo>,
        sns_links: ModalListWrapper<SnsLink, 8>,
        content: ModalWrapper<PageContent>,
    },
    News {
        basic_info: ModalWrapper<BasicInfo>,
        authors: ModalWrapper<Authors>,
        content: ModalWrapper<PageContent>,
        thumbnail: ModalWrapper<Thumbnail>,
        sns_links: ModalListWrapper<SnsLink, 8>,
    },
    Song {
        basic_info: ModalWrapper<BasicInfo>,
        authors: ModalWrapper<Authors>,
        duration: ModalWrapper<TrackDuration>,
        content: ModalWrapper<PageContent>,
        thumbnail: ModalWrapper<Thumbnail>,
        sns_links: ModalListWrapper<SnsLink, 8>,
    },
}

#[derive(Debug)]
pub enum ModalWrapper<T>
where
    T: FromModal,
{
    Unset,
    Ok {
        input: T::Input,
        value: T,
    },
    Bad {
        input: T::Input,
        error: Box<anyhow::Error>,
    },
}

impl<T> ModalWrapper<T>
where
    T: FromModal,
{
    pub fn id() -> &'static str {
        T::ID
    }

    pub fn label() -> &'static str {
        T::LABEL
    }

    pub fn required() -> Required {
        T::REQUIRED
    }

    pub fn create_discord_button(&self) -> CreateButton<'static> {
        let status = match &self {
            ModalWrapper::Unset => match Self::required() {
                Required::Yes => "‼️",
                Required::Recommended => "❗",
                Required::Optional => "ℹ️",
            },
            ModalWrapper::Ok { .. } => "✅",
            ModalWrapper::Bad { .. } => "❌",
        };

        CreateButton::new(Cow::Borrowed(Self::id()))
            .label(Cow::Borrowed(Self::label()))
            .emoji(ReactionType::Unicode(FixedString::from_static_trunc(
                status,
            )))
    }

    pub fn item_status(&self) -> String {
        let status = match &self {
            ModalWrapper::Unset => "未定義値(ご入力おねがいします)".to_string(),
            ModalWrapper::Ok { .. } => "良好".to_string(),
            ModalWrapper::Bad { error, .. } => format!("エラーが発生しました: {:?}", error),
        };

        format!(
            r#"
- {}の状態: {}のアイテム、今は{}
            "#,
            Self::label(),
            Self::required(),
            status
        )
    }
}

pub struct ModalListWrapper<T, const N: usize>
where
    T: FromListModal<N>,
{
    items: Vec<Result<T, Error>>,
    previous_inputs: Vec<Option<T::AddOrIndividualInput>>,
}

impl<T, const N: usize> ModalListWrapper<T, N>
where
    T: FromListModal<N>,
{
    pub fn add_id() -> &'static str {
        T::ADD_MODAL_ID
    }
    pub fn add_label() -> &'static str {
        T::ADD_MODAL_LABEL
    }
    pub fn remove_id() -> &'static str {
        T::REMOVE_MODAL_ID
    }
    pub fn remove_label() -> &'static str {
        T::REMOVE_MODAL_LABEL
    }
    pub fn item_ids() -> &'static [&'static str; N] {
        &T::ITEM_IDS
    }
    pub fn individual_label() -> &'static str {
        T::INDIVIDUAL_LABEL_PREFIX
    }
    pub fn required() -> Required {
        T::REQUIRED
    }

    pub fn create_button_list(&self) -> Vec<CreateActionRow<'_>> {
        let items_iter = self.items.iter();
        let item_ids = Self::item_ids().iter();
        let all_buttons = items_iter
            .zip(item_ids)
            .enumerate()
            .map(|(index, (item, id))| match item {
                Ok(_) => CreateButton::new(Cow::Borrowed(*id))
                    .label(Cow::Owned(format!(
                        "{}{}",
                        Self::individual_label(),
                        index + 1
                    )))
                    .emoji(ReactionType::Unicode(FixedString::from_static_trunc("✅"))),
                Err(_) => CreateButton::new(Cow::Borrowed(*id))
                    .label(Cow::Owned(format!(
                        "{}{}",
                        Self::individual_label(),
                        index + 1
                    )))
                    .emoji(ReactionType::Unicode(FixedString::from_static_trunc("❌"))),
            })
            .collect::<Vec<CreateButton<'_>>>();

        // discord limits us to 5 buttons per row. due to this
        // we split the buttons into multiple rows of buttons
        let mut list_of_multiple_buttons = all_buttons
            .chunks(5)
            .map(|chubks| CreateActionRow::Buttons(Cow::Owned(chubks.to_vec())))
            .collect::<Vec<CreateActionRow<'_>>>();

        // _shakes fist_ if only create button fns were const...
        let default_buttons = CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(Cow::Borrowed(Self::add_id()))
                .label(Cow::Borrowed(Self::add_label()))
                .emoji(ReactionType::Unicode(FixedString::from_static_trunc("➕"))),
            CreateButton::new(Cow::Borrowed(Self::remove_id()))
                .label(Cow::Borrowed(Self::remove_label()))
                .emoji(ReactionType::Unicode(FixedString::from_static_trunc("🗑️"))),
        ]));

        list_of_multiple_buttons.push(default_buttons);
        list_of_multiple_buttons
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Required {
    Yes,
    Recommended,
    Optional,
}

impl Display for Required {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Required::Yes => "(必須)",
            Required::Recommended => "(勧奨)",
            Required::Optional => "(選択)",
        };
        write!(f, "{s}")
    }
}

trait FromModal: Sized + Clone + Debug {
    const ID: &'static str;
    const LABEL: &'static str;
    const REQUIRED: Required;

    type Input: Modal;

    fn parse(input: Self::Input) -> Result<Self, Error>;
}

trait FromListModal<const MAX_ITEMS: usize>: Sized + Clone + Debug {
    const ADD_MODAL_ID: &'static str;
    const ADD_MODAL_LABEL: &'static str;
    const REMOVE_MODAL_ID: &'static str;
    const REMOVE_MODAL_LABEL: &'static str;
    const ITEM_IDS: [&'static str; MAX_ITEMS];
    const INDIVIDUAL_LABEL_PREFIX: &'static str;
    const REQUIRED: Required;

    type AddOrIndividualInput: Modal;
    type DeleteInput: Modal;

    fn parse_individual(input: Self::AddOrIndividualInput) -> Result<Self, Error>;

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error>;
}

fn check_attachmemt_is_picture_and_under_flimit(attachment: &Attachment) -> Result<(), Error> {
    if let None = attachment.dimensions() {
        return Err(Error::msg("Attachment is not an image!"));
    }

    if attachment.size > FILE_SIZE_LIMIT_BYTES {
        return Err(Error::msg(format!(
            "File size was larger than configured maximum {FILE_SIZE_LIMIT_BYTES}"
        )));
    }

    return Ok(());
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "ページ基本情報"]
#[text_display = "マニュアル:　"]
pub struct BasicInfoModal {
    #[name = "（必須）題目"]
    #[max_length = 50]
    #[min_length = 1]
    pub title: FixedString<u16>,
    #[name = "（必須）まとめ説明"]
    #[max_length = 150]
    #[min_length = 1]
    pub short: FixedString<u16>,
    #[name = "（必須）日付（RFC 3339)"]
    #[description = "年年年年-月月-日日 （例:　2026-07-31)"]
    pub date: FixedString<u16>,
}
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct BasicInfo {
    pub title: String,
    pub short_description: String,
    pub date: NaiveDate,
}

impl FromModal for BasicInfo {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO;
    const LABEL: &'static str = "基本情報";
    const REQUIRED: Required = Required::Yes;

    type Input = BasicInfoModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        let date = NaiveDate::parse_from_str(&input.date, "%Y-%m-%d")?;
        Ok(BasicInfo {
            title: input.title.into_string(),
            short_description: input.short.into_string(),
            date,
        })
    }
}

fn parse_authors(authors_str: &str) -> Result<Vec<String>, Error> {
    authors_str
        .split(",")
        .map(str::trim)
        .map(|x| match x.chars().all(|c| c.is_ascii_alphanumeric()) {
            true => Ok(x.to_string()),
            false => Err(Error::msg("author parsing failed: bad character")),
        })
        .collect::<Result<Vec<String>, Error>>()
}

fn parse_additional_authors(authors_str: &str) -> Vec<String> {
    authors_str
        .split(',')
        .map(str::trim)
        .map(|x| clean(x))
        .collect::<Vec<String>>()
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "作成者情報"]
#[text_display = "マニュアル:　"]
pub struct AuthorsModal {
    #[name = "メンバーページある作成者"]
    #[description = "名は\",\"で分離して入力してください。空白文字は無視されます。英語字幕のみです。（例: mitsumori, knoeze)"]
    pub authors: FixedString<u16>,
    #[name = "メンバーページない作成者"]
    #[description = "名は\",\"で分離して入力してください。空白文字は無視されます。（例:　リリイ・シュシュ, 金魚光線, Maurice Ravel）"]
    pub additional_authors: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Authors {
    pub authors: Vec<String>,
    pub additional_authors: Vec<String>,
}

impl FromModal for Authors {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS;
    const LABEL: &'static str = "作成者情報";
    const REQUIRED: Required = Required::Yes;

    type Input = AuthorsModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        let authors = parse_authors(&input.authors)?;
        let additional_authors = parse_additional_authors(&input.additional_authors);

        Ok(Authors {
            authors,
            additional_authors,
        })
    }
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "ページ内容"]
#[text_display = "マニュアル:　"]
pub struct PageContentModal {
    #[name = "（ページの内容物（例:　曲の歌詞）"]
    pub content: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct PageContent {
    pub content: String,
}

impl FromModal for PageContent {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_CONTENT;
    const LABEL: &'static str = "ページ内容";
    const REQUIRED: Required = Required::Recommended;

    type Input = PageContentModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        let options = Options::all();
        for event in Parser::new_ext(&input.content, options) {
            if let Event::End(_) = event {
                return Ok(PageContent {
                    content: input.content.into_string(),
                });
            }
        }
        Err(Error::msg("Bad Markdown"))
    }
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "主要リンク"]
#[text_display = "マニュアル:　"]
pub struct MainLinkModal {
    #[name = "主要リンク先"]
    pub link: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct MainLink {
    pub url: Url,
}

impl FromModal for MainLink {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_LINK;
    const LABEL: &'static str = "主要リンク";
    const REQUIRED: Required = Required::Recommended;

    type Input = MainLinkModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        Ok(MainLink {
            url: Url::parse(&input.link)?,
        })
    }
}

#[derive(Clone, Debug, Modal)]
#[name = "サムネイル"]
#[text_display = "マニュアル:　"]
pub struct ThumbnailModal {
    #[file_upload]
    #[file_types("image")]
    #[name = "サムネイルファイル（一つのファイルのみ)"]
    pub image: FixedArray<Attachment, u32>,
    #[name = "メンバーページある作成者"]
    #[description = "名は\",\"で分離して入力してください。英語字幕のみです。（例: mitsumori, knoeze, seeyoumayday)"]
    pub authors: FixedString<u16>,
    #[name = "メンバーページない作成者"]
    #[description = "名は\",\"で分離して入力してください。（例:　リリイ・シュシュ, 金魚光線, Maurice Ravel）"]
    pub additional_authors: FixedString<u16>,
}

#[derive(Clone, Debug)]
pub struct Thumbnail {
    pub illustration: Illustration,
    pub attachment: Attachment,
}

impl FromModal for Thumbnail {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL;
    const LABEL: &'static str = "サムネイル設定";
    const REQUIRED: Required = Required::Optional;

    type Input = ThumbnailModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        if input.image.len() != 1 {
            return Err(Error::msg("Needs exactly one image!"));
        }

        let attachment = input.image[0].clone();

        check_attachmemt_is_picture_and_under_flimit(&attachment)?;

        let illustrators = parse_authors(&input.authors)?;
        let additional_illustrators = parse_additional_authors(&input.additional_authors);

        let illustration = process_attachment(
            &attachment,
            illustrators,
            additional_illustrators,
            Some("Thumbnail".to_string()),
        )?;

        Ok(Thumbnail {
            illustration,
            attachment,
        })
    }
}

fn process_attachment(
    attachment: &Attachment,
    illustrators: Vec<String>,
    additional_illustrators: Vec<String>,
    description: Option<String>,
) -> Result<Illustration, Error> {
    check_attachmemt_is_picture_and_under_flimit(attachment)?;

    let filename = &attachment.filename;
    let attachment_id_b64 =
        base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&attachment.id.get().to_be_bytes());
    let destination_filename = format!("images/{attachment_id_b64}-{filename}");

    let illustration = Illustration {
        image: destination_filename,
        illustrators,
        additional_illustrators,
        description,
    };

    Ok(illustration)
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "曲の長さ入力"]
pub struct DurationModal {
    #[max_length = 5]
    #[min_length = 1]
    #[name = "（必須）アルバムトラック長さ"]
    #[description = "MMm SSs　（例:　３分２５秒は 3m 25s)"]
    pub length_mmss: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackDuration(pub FancyDuration<Duration>);

impl FromModal for TrackDuration {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_DURATION;
    const LABEL: &'static str = "曲の長さ入力";
    const REQUIRED: Required = Required::Yes;

    type Input = DurationModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        Ok(TrackDuration(FancyDuration::parse(&input.length_mmss)?))
    }
}

#[derive(Clone, Debug, Modal)]
#[name = "メンバー基本情報"]
#[text_display = ""]
pub struct MemberBasicInfoModal {
    #[name = "（必須）活動名"]
    #[max_length = 50]
    #[min_length = 1]
    #[description = "\",\"と空白文字は使えないです。"]
    pub name: FixedString<u16>,
    #[name = "（必須）サイトascii名"]
    #[max_length = 50]
    #[min_length = 1]
    #[description = "英語字幕のみ使えます。他人と重複名前を使用は不可"]
    pub ascii_name: FixedString<u16>,
    #[name = "（必須）まとめ説明"]
    #[max_length = 150]
    #[min_length = 1]
    pub short: FixedString<u16>,
    #[file_upload]
    #[file_types("image")]
    #[name = "（必須）メンバープロファイル写真"]
    pub picture: FixedArray<Attachment, u32>,
}

#[derive(Clone, Debug)]
pub struct MemberBasicInfo {
    pub name: String,
    pub ascii_name: String,
    pub short: String,
    pub picture: Attachment,
}

impl FromModal for MemberBasicInfo {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO;
    const LABEL: &'static str = "メンバー基本情報";
    const REQUIRED: Required = Required::Yes;

    type Input = MemberBasicInfoModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        input
            .ascii_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
            .ok_or(Error::msg("Illegal character in `ascii_name`"))?;

        if input.name.contains(",") || input.name.contains("、") {
            return Err(Error::msg("Illegal character in `name`"));
        }

        if input.picture.len() != 1 {
            return Err(Error::msg("Only 1 picture allowed!"));
        }

        let attachment = input.picture[0].clone();

        check_attachmemt_is_picture_and_under_flimit(&attachment)?;

        Ok(MemberBasicInfo {
            name: input.name.into_string(),
            ascii_name: input.ascii_name.into_string(),
            short: input.short.into_string(),
            picture: attachment,
        })
    }
}

#[derive(Clone, Debug, Modal)]
#[name = "メンバー追加情報"]
#[text_display = ""]
pub struct ExtendedMemberBasicInfoModal {
    #[name = "（選択）学部"]
    #[max_length = 20]
    pub department: Option<FixedString<u16>>,
    #[name = "（選択）サークル内の役割"]
    #[max_length = 20]
    #[description = "勝手に設置は禁止です。特の役割がない場合、空白で残してください"]
    pub position: Option<FixedString<u16>>,
    #[name = "（選択）サークル入会年度"]
    #[max_length = 4]
    pub entry: Option<FixedString<u16>>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ExtendedMemberBasicInfo {
    pub department: Option<String>,
    pub position: Option<String>,
    pub entry: Option<String>,
}

impl FromModal for ExtendedMemberBasicInfo {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO;
    const LABEL: &'static str = "追加メンバー情報入力";
    const REQUIRED: Required = Required::Optional;

    type Input = ExtendedMemberBasicInfoModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        Ok(ExtendedMemberBasicInfo {
            department: input.department.map(FixedString::into_string),
            position: input.position.map(FixedString::into_string),
            entry: input.entry.map(FixedString::into_string),
        })
    }
}

// from here, modals that can have multiples (sns link, track, illusts)
//

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "SNSリンク追加"]
#[text_display = "マニュアル:　"]
pub struct SnsLinksModal {
    pub link: FixedString<u16>,
}

// from here, modals that can have multiples (sns link, track, illusts)
#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "SNSリンク消去"]
#[text_display = "マニュアル:　"]
pub struct DeleteSnsLinksModal {
    #[name = "（必須）消去するリンクの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SnsLink {
    pub url: Url,
}

impl FromListModal<8> for SnsLink {
    const ADD_MODAL_ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK;

    const ADD_MODAL_LABEL: &'static str = "SNSリンク追加";

    const REMOVE_MODAL_ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK;

    const REMOVE_MODAL_LABEL: &'static str = "SNSリンク消去";

    const ITEM_IDS: [&'static str; 8] = [
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_1,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_2,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_3,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_4,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_5,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_6,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_7,
        ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINK_8,
    ];

    const INDIVIDUAL_LABEL_PREFIX: &'static str = "SNSリンク:　第";

    const REQUIRED: Required = Required::Optional;

    type AddOrIndividualInput = SnsLinksModal;

    type DeleteInput = DeleteSnsLinksModal;

    fn parse_individual(input: Self::AddOrIndividualInput) -> Result<Self, Error> {
        Ok(SnsLink {
            url: Url::parse(&input.link)?,
        })
    }

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error> {
        Ok(input.number.parse::<usize>()?)
    }
}

#[derive(Clone, Debug, Modal)]
#[name = "イラスト追加"]
pub struct IllustrationModal {
    #[name = "このイラストの題名"]
    #[max_length = 50]
    #[min_length = 1]
    pub title: FixedString<u16>,
    #[name = "このイラストの説明"]
    #[max_length = 50]
    #[min_length = 1]
    pub description: FixedString<u16>,
    #[file_upload]
    #[file_types("image")]
    #[name = "ファイル (JPG, PNG, WEBPのみ)"]
    pub image: FixedArray<Attachment, u32>,
    #[name = "メンバーページある作成者"]
    #[description = "名は\",\"で分離して入力してください。英語字幕のみです。（例: mitsumori, knoeze, seeyoumayday)"]
    pub authors: FixedString<u16>,
    #[name = "メンバーページない作成者"]
    #[description = "名は\",\"で分離して入力してください。（例:　リリイ・シュシュ, 金魚光線, Maurice Ravel）"]
    pub additional_authors: FixedString<u16>,
}

#[derive(Clone, Debug, Modal)]
#[name = "イラスト消去"]
#[text_display = "この編集から追加したイラストを消す"]
pub struct DeleteIllustrationModal {
    #[name = "（必須）消去するイラストの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug)]
pub struct TitleAndIllustraion {
    pub title: String,
    pub illustration: Illustration,
    pub attachment: Attachment,
}

impl FromListModal<5> for TitleAndIllustraion {
    const ADD_MODAL_ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION;
    const ADD_MODAL_LABEL: &'static str = "イラスト追加";

    const REMOVE_MODAL_ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION;
    const REMOVE_MODAL_LABEL: &'static str = "イラスト消去";

    const ITEM_IDS: [&'static str; 5] = [
        ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_1,
        ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_2,
        ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_3,
        ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_4,
        ZVEZDOCHKA_TICKET_COMPONENT_ILLUSTRATION_5,
    ];

    const INDIVIDUAL_LABEL_PREFIX: &'static str = "イラスト:　第";

    const REQUIRED: Required = Required::Optional;

    type AddOrIndividualInput = IllustrationModal;

    type DeleteInput = DeleteIllustrationModal;

    fn parse_individual(input: Self::AddOrIndividualInput) -> Result<Self, Error> {
        if input.image.len() != 1 {
            return Err(Error::msg("Needs exactly one image!"));
        }

        let attachment = input.image[0].clone();

        check_attachmemt_is_picture_and_under_flimit(&attachment)?;

        let illustrators = parse_authors(&input.authors)?;
        let additional_illustrators = parse_additional_authors(&input.additional_authors);

        let illustration = process_attachment(
            &attachment,
            illustrators,
            additional_illustrators,
            Some(input.description.into_string()),
        )?;

        Ok(TitleAndIllustraion {
            title: input.title.into_string(),
            illustration,
            attachment,
        })
    }

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error> {
        Ok(input.number.parse::<usize>()?)
    }
}

// album tracks

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "アルバムトラックずつ情報"]
pub struct AlbumTrackModal {
    #[name = "（必須）アルバムトラック題名"]
    #[max_length = 50]
    #[min_length = 1]
    pub title: FixedString<u16>,
    #[max_length = 5]
    #[min_length = 1]
    #[name = "（必須）アルバムトラック長さ"]
    #[description = "MMm SS　（例:　３分２５秒は 3m 25)"]
    pub length_mmss: FixedString<u16>,
    #[name = "メンバーページある作成者"]
    #[description = "名は\",\"で分離して入力してください。空白文字は無視されます。英語字幕のみです。（例: mitsumori, knoeze)"]
    pub authors: FixedString<u16>,
    #[name = "メンバーページない作成者"]
    #[description = "名は\",\"で分離して入力してください。空白文字は無視されます。（例:　リリイ・シュシュ, 金魚光線, Maurice Ravel）"]
    pub additional_authors: FixedString<u16>,
}

#[derive(Clone, Debug, Modal)]
#[name = "トラック消去"]
#[text_display = "この編集から追加したトラックを消す"]
pub struct DeleteTrackModal {
    #[name = "（必須）消去するトラックの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AlbumTitleAndTrack {
    pub title: String,
    pub track: Track,
}

impl FromListModal<20> for AlbumTitleAndTrack {
    const ADD_MODAL_ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK;
    const ADD_MODAL_LABEL: &'static str = "トラック追加";

    const REMOVE_MODAL_ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK;
    const REMOVE_MODAL_LABEL: &'static str = "トラック消去";

    const ITEM_IDS: [&'static str; 20] = [
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_1,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_2,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_3,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_4,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_5,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_6,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_7,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_8,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_9,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_10,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_11,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_12,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_13,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_14,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_15,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_16,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_17,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_18,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_19,
        ZVEZDOCHKA_TICKET_COMPONENT_TRACK_20,
    ];

    const INDIVIDUAL_LABEL_PREFIX: &'static str = "トラック:　第";

    const REQUIRED: Required = Required::Optional;

    type AddOrIndividualInput = AlbumTrackModal;
    type DeleteInput = DeleteTrackModal;

    fn parse_individual(input: Self::AddOrIndividualInput) -> Result<Self, Error> {
        let title = input.title.to_string();
        let duration = FancyDuration::parse(&input.length_mmss)?;
        let authors = parse_authors(&input.authors)?;
        let additional_authors = parse_additional_authors(&input.additional_authors);

        Ok(AlbumTitleAndTrack {
            title,
            track: Track {
                authors,
                additional_authors,
                duration,
            },
        })
    }

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error> {
        Ok(input.number.parse::<usize>()?)
    }
}

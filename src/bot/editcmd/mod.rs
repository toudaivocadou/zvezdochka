use std::{borrow::Cow, fmt::Display, sync::Arc, time::Duration};

use ammonia::clean;
use anyhow::Error;
use base64::Engine;
use chrono::{NaiveDate, Utc};
use fancy_duration::FancyDuration;
use poise::{
    Modal,
    serenity_prelude::{
        Attachment, ChannelId, CreateActionRow, CreateButton, CreateComponent, CreateTextDisplay,
        EditMessage, GenericChannelId, Http, MessageId,
        small_fixed_array::{FixedArray, FixedString},
    },
};
use pulldown_cmark::{Event, Options, Parser};
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
            ZVEZDOCHKA_TICKET_COMPONENT_ILLUST_IDX, ZVEZDOCHKA_TICKET_COMPONENT_LINK,
            ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION, ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK,
            ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK, ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINKS_IDX,
            ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL, ZVEZDOCHKA_TICKET_COMPONENT_TRACK_IDX,
        },
    },
    site::album::{Illustration, Track},
};

mod album;
mod artist;
mod news;
mod song;

pub const FILE_SIZE_LIMIT_BYTES: u32 = 10485760; // 10 mb

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

pub enum WorkStateData {
    Album {
        basic_info: Option<BasicInfo>,
        authors: Option<Authors>,
        content: Option<PageContent>,
        link: Option<MainLink>,
        sns_links: Option<Vec<SnsLink>>,
        thumbnail: Option<Thumbnail>,
        illustrations: Option<Vec<Illustration>>,
        tracks: Option<Vec<AlbumTitleAndTrack>>,
    },
    Artist {
        artist_info: Option<MemberBasicInfo>,
        additional_artist_info: Option<ExtendedMemberBasicInfo>,
        sns_links: Option<Vec<SnsLink>>,
        content: Option<PageContent>,
    },
    News {
        basic_info: Option<BasicInfo>,
        authors: Option<Authors>,
        content: Option<PageContent>,
        thumbnail: Option<Thumbnail>,
        sns_links: Option<Vec<SnsLink>>,
    },
    Song {
        basic_info: Option<BasicInfo>,
        authors: Option<Authors>,
        duration: Option<TrackDuration>,
        content: Option<PageContent>,
        thumbnail: Option<Thumbnail>,
        sns_links: Option<Vec<SnsLink>>,
    },
}

pub enum ErrorState {
    Unset,
    Ok,
    Bad,
}

pub struct ModalWrapper<T>
where
    T: FromModal,
{
    value: T,
    previous_modal_input: Option<T::Input>,
    state: ErrorState,
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

trait FromModal: Sized {
    const ID: &'static str;
    const LABEL: &'static str;
    const REQUIRED: Required;

    type Input;

    fn parse(input: Self::Input) -> Result<Self, Error>;
}

impl<T> FromModal for Option<T>
where
    T: FromModal,
{
    const ID: &'static str = T::ID;

    const LABEL: &'static str = T::LABEL;

    const REQUIRED: Required = T::REQUIRED;

    type Input = T::Input;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        Ok(Some(T::parse(input)?))
    }
}

impl<T> FromModal for &Option<T>
where
    T: FromModal,
{
    const ID: &'static str = T::ID;

    const LABEL: &'static str = T::LABEL;

    const REQUIRED: Required = T::REQUIRED;

    type Input = T::Input;

    // yes i do understand this is a crime but please bear with me i just need to access the constants :c
    fn parse(_input: Self::Input) -> Result<Self, Error> {
        return Err(Error::msg(
            "Because Vorkuta-5 does not exist. People from Vorkuta-5 do not exist.",
        ));
    }
}

fn id_of_type<T: FromModal>(_id: &T) -> &'static str {
    T::ID
}

fn label_of_type<T: FromModal>(_id: &T) -> &'static str {
    T::LABEL
}

fn required_of_type<T: FromModal>(_id: &T) -> Required {
    T::REQUIRED
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

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd)]
#[name = "SNSリンク追加"]
#[text_display = "マニュアル:　"]
pub struct SnsLinksModal {
    pub link: FixedString<u16>,
}

pub struct SnsLink {
    pub url: Url,
}

impl FromModal for SnsLink {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK;
    const LABEL: &'static str = "SNSリンク追加";
    const REQUIRED: Required = Required::Optional;

    type Input = SnsLinksModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
        Ok(SnsLink {
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

pub struct TitleAndIllustraion {
    pub title: String,
    pub illustration: Illustration,
    pub attachment: Attachment,
}

impl FromModal for TitleAndIllustraion {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION;
    const LABEL: &'static str = "イラスト追加";
    const REQUIRED: Required = Required::Optional;

    type Input = IllustrationModal;

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
            Some(input.description.into_string()),
        )?;

        Ok(TitleAndIllustraion {
            title: input.title.into_string(),
            illustration,
            attachment,
        })
    }
}

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

pub struct AlbumTitleAndTrack {
    pub title: String,
    pub track: Track,
}

impl FromModal for AlbumTitleAndTrack {
    const ID: &'static str = ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK;
    const LABEL: &'static str = "アルバムトラック追加";
    const REQUIRED: Required = Required::Optional;

    type Input = AlbumTrackModal;

    fn parse(input: Self::Input) -> Result<Self, Error> {
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

#[derive(Clone, Debug, Modal)]
#[name = "イラスト消去"]
#[text_display = "この編集から追加したイラストを消す"]
pub struct DeleteIllustrationModal {
    #[name = "（必須）消去するイラストの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug, Modal)]
#[name = "トラック消去"]
#[text_display = "この編集から追加したトラックを消す"]
pub struct DeleteTrackModal {
    #[name = "（必須）消去するトラックの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

macro_rules! button_list_ {
    () => {};
}

pub async fn discard_current_work(
    http: Arc<Http>,
    data: Arc<Data>,
    channel: ChannelId,
) -> Result<(), Error> {
    if let Some((_, Some(workstate))) = data.work_states.remove(&channel) {
        EditMessage::new()
            .components(&[CreateComponent::TextDisplay(CreateTextDisplay::new(
                "この編集先は削除されました。",
            ))])
            .execute(
                http,
                GenericChannelId::new(channel.get()),
                workstate.current_work_message,
                None,
            )
            .await?;
    }
    return Ok(());
}

pub async fn check_channel_is_work_channel(
    data: Arc<Data>,
    channel: ChannelId,
) -> Result<(), Error> {
    data.database
        .is_channel_id_a_ticket_channel(channel)
        .await?
        .ok_or(Error::msg("このチャネルは編集チャネルではありません。"))
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
enum WorkDataStatus {
    Good,
    Bad,
    None,
}

impl WorkDataStatus {
    pub fn is_good(&self) -> bool {
        match self {
            WorkDataStatus::Good => true,
            WorkDataStatus::Bad | WorkDataStatus::None => false,
        }
    }
}

impl Display for WorkDataStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WorkDataStatus::Good => "✅",
            WorkDataStatus::Bad => "❌",
            WorkDataStatus::None => "❓",
        };
        write!(f, "{s}")
    }
}

pub fn create_work_components(
    work_state: &WorkStateData,
) -> Result<Cow<'_, [CreateComponent]>, Error> {
    let expiry_time = Utc::now().timestamp() + 800;

    match work_state {
        WorkStateData::Album {
            basic_info,
            authors,
            content,
            link,
            sns_links,
            thumbnail,
            illustrations,
            tracks,
        } => {
            let basic_info = match basic_info {
                Some(_) => todo!(),
                None => todo!(),
            };
        }
        WorkStateData::Artist {
            artist_info,
            sns_links,
            content,
            additional_artist_info,
        } => todo!(),
        WorkStateData::News {
            basic_info,
            authors,
            content,
            sns_links,
            thumbnail,
        } => todo!(),
        WorkStateData::Song {
            basic_info,
            authors,
            duration,
            content,
            sns_links,
            thumbnail,
        } => todo!(),
    }
}

fn basic_info_create_action_row(work_state_data: &WorkStateData) -> CreateActionRow<'static> {
    match work_state_data {
        WorkStateData::Album {
            basic_info,
            authors,
            content,
            link,
            sns_links,
            thumbnail,
            illustrations,
            tracks,
        } => CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(id_of_type(basic_info)).label(format!(
                "{} {}",
                required_of_type(basic_info),
                label_of_type(basic_info)
            )),
            CreateButton::new(id_of_type(authors)).label(format!(
                "{} {}",
                required_of_type(authors),
                label_of_type(authors)
            )),
            CreateButton::new(id_of_type(content)).label(format!(
                "{} {}",
                required_of_type(content),
                label_of_type(content)
            )),
            CreateButton::new(id_of_type(sns_links)).label(format!(
                "{} {}",
                required_of_type(sns_links),
                label_of_type(sns_links)
            )),
            CreateButton::new(id_of_type(thumbnail)).label(format!(
                "{} {}",
                required_of_type(thumbnail),
                label_of_type(thumbnail)
            )),
        ])),
        WorkStateData::Artist { .. } => CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_ARTISTINFO)
                .label("メンバー基本情報編集（必須）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_EXT_ARTISTINFO)
                .label("追加メンバー情報編集（選択）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_CONTENT)
                .label("メンバーページ内容編集（選択）"),
        ])),
        WorkStateData::News { .. } => CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO).label("基本情報編集（必須）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS).label("作成者情報編集（選択）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_CONTENT)
                .label("ニュースポスト内容（選択）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL).label("サムネイル（選択）"),
        ])),
        WorkStateData::Song { .. } => CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_BASICINFO).label("基本情報編集（必須）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_AUTHORS).label("作成者情報編集（必須）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_DURATION).label("曲の長さ（必須）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_CONTENT)
                .label("曲の歌詞・追加情報・説明（選択）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_THUMBNAIL).label("サムネイル（選択）"),
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_LINK).label("曲の視聴リンク（選択）"),
        ])),
    }
}

fn create_sns_links_row(
    sns_links: &Option<Vec<SnsLinksModal>>,
) -> Result<CreateActionRow<'static>, Error> {
    match sns_links {
        Some(sns_links) => {
            if sns_links.len() > 10 {
                return Err(Error::msg(
                    "Exceeded maximum number of SNS Links! (max: 10)",
                ));
            }

            let mut sns_buttons = vec![
                CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK).label("SNSリンク追加"),
                CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_DELETE_SNSLINK)
                    .label("SNSリンク除去"),
            ];

            for (idx, _) in sns_links.iter().enumerate() {
                sns_buttons.push(
                    CreateButton::new(format!("{ZVEZDOCHKA_TICKET_COMPONENT_SNS_LINKS_IDX}{idx}"))
                        .label(format!("第{idx}SNSリンク編集")),
                );
            }

            Ok(CreateActionRow::Buttons(Cow::Owned(sns_buttons)))
        }
        None => Ok(CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_NEW_SNSLINK).label("SNSリンク追加"),
        ]))),
    }
}

fn album_create_songs_row(
    song_modals: &Option<Vec<AlbumTrackModal>>,
) -> Result<CreateActionRow<'static>, Error> {
    match song_modals {
        Some(album_tracks) => {
            if album_tracks.len() > 25 {
                return Err(Error::msg(
                    "Exceeded maximum number of Album Tracks! (max: 25)",
                ));
            }

            let mut tracks = vec![
                CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK)
                    .label("アルバムトラック追加"),
                CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_DELETE_TRACK)
                    .label("アルバムトラック除去"),
            ];

            for (idx, _) in album_tracks.iter().enumerate() {
                tracks.push(
                    CreateButton::new(format!("{ZVEZDOCHKA_TICKET_COMPONENT_TRACK_IDX}{idx}"))
                        .label(format!("第{idx}アルバムトラック編集")),
                );
            }

            Ok(CreateActionRow::Buttons(Cow::Owned(tracks)))
        }
        None => Ok(CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_NEW_TRACK).label("アルバムトラック追加"),
        ]))),
    }
}

fn album_create_illustrations_row(
    illust_modals: &Option<Vec<IllustrationModal>>,
) -> Result<CreateActionRow<'static>, Error> {
    match illust_modals {
        Some(illust_modals) => {
            if illust_modals.len() > 10 {
                return Err(Error::msg(
                    "Exceeded maximum number of Illustrations! (max: 10)",
                ));
            }

            let mut illust_buttons = vec![
                CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION)
                    .label("イラスト追加"),
                CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_DELETE_ILLUSTRATION)
                    .label("イラスト除去"),
            ];

            for (idx, _) in illust_modals.iter().enumerate() {
                illust_buttons.push(
                    CreateButton::new(format!("{ZVEZDOCHKA_TICKET_COMPONENT_ILLUST_IDX}{idx}"))
                        .label(format!("第{idx}アルバムトラック編集")),
                );
            }

            Ok(CreateActionRow::Buttons(Cow::Owned(illust_buttons)))
        }
        None => Ok(CreateActionRow::Buttons(Cow::Owned(vec![
            CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_NEW_ILLUSTRATION).label("イラスト追加"),
        ]))),
    }
}

fn confirm_cancel_row() -> CreateActionRow<'static> {
    CreateActionRow::Buttons(Cow::Owned(vec![
        CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_CANCEL_WORK).label("キャンセル"),
        CreateButton::new(ZVEZDOCHKA_TICKET_COMPONENT_CONFIRM_WORK).label("確定"),
    ]))
}

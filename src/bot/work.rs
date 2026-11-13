use ammonia::clean;
use anyhow::Error;
use chrono::NaiveDate;
use fancy_duration::FancyDuration;
use poise::{
    Modal,
    serenity_prelude::{
        ButtonStyle, CreateActionRow, CreateButton, CreateComponent, CreateSeparator,
        CreateTextDisplay, small_fixed_array::FixedString,
    },
};
use pulldown_cmark::{Event, Options, Parser};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::{borrow::Cow, fmt::Display, time::Duration};
use url::Url;

use crate::{
    bot::{
        Work,
        utils::{
            ComponentInteractionTypes, EditConfirmCancel, ListInteraction, MultiInteractType,
            MultipleEditComponents, PageTurners, PathSection, SingularEditComponents,
            check_file_exists,
        },
    },
    site::{
        album::{Illustration, Track},
        namemap::NameMap,
    },
};

pub const DISCORD_MESSAGE_NO_TAKEBACKSIES: &'static str = "※入力を**クリアすることはできません**。選択の入力もクリアができません。***慎重にボタンをクリックするべき***";
pub const DISCORD_MESSAGE_FRONTMATTER: &'static str = r#"
以下のボタンを使えばサイトのページが編集できます。
編集作業が完成になると「編集確定」をクリックしてください。
編集作業のキャンセルは「編集をキャンセル」をクリックしたらできます。

問題があればサイト担当者に届けてください。
"#;
pub const DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION: &'static str = r#"
### 状態の説明
- 🟩: この入力は良好
- ⬛: この入力は基本値
- 🟥: この入力は不良
"#;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlbumWorkStatePage {
    BasicInfo,
    Tracks,
    SnsLink,
    Illustrations,
}

impl AlbumWorkStatePage {
    pub fn to_buttons(&self) -> Vec<CreateButton<'_>> {
        let needed = match self {
            AlbumWorkStatePage::BasicInfo => {
                vec![
                    ComponentInteractionTypes::PageTurners(PageTurners::ToAlbumTracks),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToSnsLinks),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToIllustrations),
                ]
            }
            AlbumWorkStatePage::Tracks => {
                vec![
                    ComponentInteractionTypes::PageTurners(PageTurners::ToInfo),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToSnsLinks),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToIllustrations),
                ]
            }
            AlbumWorkStatePage::SnsLink => {
                vec![
                    ComponentInteractionTypes::PageTurners(PageTurners::ToInfo),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToAlbumTracks),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToIllustrations),
                ]
            }
            AlbumWorkStatePage::Illustrations => {
                vec![
                    ComponentInteractionTypes::PageTurners(PageTurners::ToInfo),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToAlbumTracks),
                    ComponentInteractionTypes::PageTurners(PageTurners::ToSnsLinks),
                ]
            }
        };
        needed.into_iter().map(|cit| cit.to_button()).collect()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OtherWorkStatePage {
    BasicInfo,
    SnsLink,
}

impl OtherWorkStatePage {
    pub fn to_label(&self) -> ComponentInteractionTypes {
        match self {
            OtherWorkStatePage::BasicInfo => {
                ComponentInteractionTypes::PageTurners(PageTurners::ToInfo)
            }
            OtherWorkStatePage::SnsLink => {
                ComponentInteractionTypes::PageTurners(PageTurners::ToSnsLinks)
            }
        }
    }
}

pub fn preparing_work_component() -> Cow<'static, [CreateComponent<'static>]> {
    Cow::Owned(vec![CreateComponent::TextDisplay(CreateTextDisplay::new(
        Cow::Borrowed("作業準備中...少々お待ち下さい..."),
    ))])
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkData {
    Album {
        page: AlbumWorkStatePage,
        basic_info: ModalWrapper<BasicInfo>,
        authors: ModalWrapper<Authors>,
        content: ModalWrapper<PageContent>,
        link: ModalWrapper<MainLink>,
        thumbnail: ModalWrapper<Thumbnail>,
        illustrations: ModalListWrapper<TitleAndIllustraion>,
        sns_links: ModalListWrapper<SnsLink>,
        tracks: ModalListWrapper<AlbumTitleAndTrack>,
    },
    Artist {
        page: OtherWorkStatePage,
        artist_info: ModalWrapper<MemberBasicInfo>,
        additional_artist_info: ModalWrapper<ExtendedMemberBasicInfo>,
        content: ModalWrapper<PageContent>,
        sns_links: ModalListWrapper<SnsLink>,
    },
    News {
        page: OtherWorkStatePage,
        basic_info: ModalWrapper<BasicInfo>,
        authors: ModalWrapper<Authors>,
        content: ModalWrapper<PageContent>,
        thumbnail: ModalWrapper<Thumbnail>,
        sns_links: ModalListWrapper<SnsLink>,
    },
    Song {
        page: OtherWorkStatePage,
        basic_info: ModalWrapper<BasicInfo>,
        authors: ModalWrapper<Authors>,
        duration: ModalWrapper<TrackDuration>,
        content: ModalWrapper<PageContent>,
        thumbnail: ModalWrapper<Thumbnail>,
        sns_links: ModalListWrapper<SnsLink>,
    },
}

impl WorkData {
    pub fn create_components(&self, filename: Option<&str>) -> Vec<CreateComponent<'_>> {
        let file_name_msg = match filename {
            Some(f) => format!("## 編集するファイル名: {f}"),
            None => format!("## 編集するファイル名: <新しいファイル>"),
        };

        let mut confirm_cancel_turnpage_buttons = vec![
            ComponentInteractionTypes::EditConfirmCancel(EditConfirmCancel::Confirm).to_button(),
            ComponentInteractionTypes::EditConfirmCancel(EditConfirmCancel::Cancel).to_button(),
        ];

        let mut components = match &self {
            WorkData::Album {
                page,
                basic_info,
                authors,
                content,
                link,
                sns_links,
                thumbnail,
                illustrations,
                tracks,
            } => {
                let mut page_turners = page.to_buttons();
                confirm_cancel_turnpage_buttons.append(&mut page_turners);

                match page {
                    AlbumWorkStatePage::BasicInfo => {
                        let status_basic_info = basic_info.item_status();
                        let status_authors = authors.item_status();
                        let status_content = content.item_status();
                        let status_link = link.item_status();
                        let status_thumbnail = thumbnail.item_status();

                        let status_short_snslink = sns_links.item_status_has_bad();
                        let status_short_illustrations = illustrations.item_status_has_bad();
                        let status_short_tracks = tracks.item_status_has_bad();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_basic_info}
{status_authors}
{status_content}
{status_link}
{status_thumbnail}

他のページの内容まとめ:
SNSリンク: {status_short_snslink}
イラスト: {status_short_illustrations}
トラック: {status_short_tracks}
                            "#
                        );

                        let button_basic_info = basic_info.create_discord_button();
                        let button_authors = authors.create_discord_button();
                        let button_content = content.create_discord_button();
                        let button_link = link.create_discord_button();
                        let button_thumbnail = thumbnail.create_discord_button();

                        vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                            CreateComponent::ActionRow(CreateActionRow::Buttons(Cow::Owned(vec![
                                button_basic_info,
                                button_authors,
                                button_content,
                                button_link,
                                button_thumbnail,
                            ]))),
                        ]
                    }
                    AlbumWorkStatePage::Tracks => {
                        let status_short_snslink = sns_links.item_status_has_bad();
                        let status_short_illustrations = illustrations.item_status_has_bad();

                        let status_tracks = tracks.items_statuses();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_tracks}

他のページの内容まとめ:
SNSリンク: {status_short_snslink}
イラスト: {status_short_illustrations}
                            "#
                        );

                        let mut status_components = vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                        ];
                        status_components.append(&mut tracks.create_button_list());
                        status_components
                    }
                    AlbumWorkStatePage::SnsLink => {
                        let status_short_tracks = tracks.item_status_has_bad();
                        let status_short_illust = illustrations.item_status_has_bad();

                        let status_snslink = sns_links.items_statuses();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_snslink}

他のページの内容まとめ:
イラスト: {status_short_illust}
トラック: {status_short_tracks}
                            "#
                        );

                        let mut status_components = vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                        ];
                        status_components.append(&mut sns_links.create_button_list());
                        status_components
                    }
                    AlbumWorkStatePage::Illustrations => {
                        let status_short_tracks = tracks.item_status_has_bad();
                        let status_short_snslinks = sns_links.item_status_has_bad();

                        let status_illustration = illustrations.items_statuses();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_illustration}

他のページの内容まとめ:
SNSリンク: {status_short_snslinks}
トラック: {status_short_tracks}
                            "#
                        );

                        let mut status_components = vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                        ];
                        status_components.append(&mut illustrations.create_button_list());
                        status_components
                    }
                }
            }
            WorkData::Artist {
                page,
                artist_info,
                additional_artist_info,
                sns_links,
                content,
            } => {
                let page_turner = page.to_label().to_button();
                confirm_cancel_turnpage_buttons.push(page_turner);

                match page {
                    OtherWorkStatePage::BasicInfo => {
                        let status_artist_info = artist_info.item_status();
                        let status_additional_artist_info = additional_artist_info.item_status();
                        let status_content = content.item_status();

                        let status_short_snslink = sns_links.item_status_has_bad();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_artist_info}
{status_additional_artist_info}
{status_content}

他のページの内容まとめ:
SNSリンク: {status_short_snslink}
                            "#
                        );

                        let button_artist_info = artist_info.create_discord_button();
                        let button_additional_artist_info =
                            additional_artist_info.create_discord_button();
                        let button_content = content.create_discord_button();

                        vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                            CreateComponent::ActionRow(CreateActionRow::Buttons(Cow::Owned(vec![
                                button_artist_info,
                                button_additional_artist_info,
                                button_content,
                            ]))),
                        ]
                    }
                    OtherWorkStatePage::SnsLink => {
                        let status_snslink = sns_links.items_statuses();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_snslink}
                            "#
                        );

                        let mut status_components = vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                        ];
                        status_components.append(&mut sns_links.create_button_list());
                        status_components
                    }
                }
            }
            WorkData::News {
                page,
                basic_info,
                authors,
                content,
                thumbnail,
                sns_links,
            } => {
                let page_turner = page.to_label().to_button();
                confirm_cancel_turnpage_buttons.push(page_turner);

                match page {
                    OtherWorkStatePage::BasicInfo => {
                        let status_basic_info = basic_info.item_status();
                        let status_authors = authors.item_status();
                        let status_content = content.item_status();
                        let status_thumbnail = thumbnail.item_status();

                        let status_short_snslink = sns_links.item_status_has_bad();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_basic_info}
{status_authors}
{status_content}
{status_thumbnail}

他のページの内容まとめ:
SNSリンク: {status_short_snslink}
                        "#
                        );

                        let button_basic_info = basic_info.create_discord_button();
                        let button_authors = authors.create_discord_button();
                        let button_content = content.create_discord_button();
                        let button_thumbnail = thumbnail.create_discord_button();

                        vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                            CreateComponent::ActionRow(CreateActionRow::Buttons(Cow::Owned(vec![
                                button_basic_info,
                                button_authors,
                                button_content,
                                button_thumbnail,
                            ]))),
                        ]
                    }
                    OtherWorkStatePage::SnsLink => {
                        let status_snslink = sns_links.items_statuses();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_snslink}
                        "#
                        );

                        let mut status_components = vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                        ];
                        status_components.append(&mut sns_links.create_button_list());
                        status_components
                    }
                }
            }
            WorkData::Song {
                page,
                basic_info,
                authors,
                duration,
                content,
                thumbnail,
                sns_links,
            } => {
                let page_turner = page.to_label().to_button();
                confirm_cancel_turnpage_buttons.push(page_turner);

                match page {
                    OtherWorkStatePage::BasicInfo => {
                        let status_basic_info = basic_info.item_status();
                        let status_authors = authors.item_status();
                        let status_duration = duration.item_status();
                        let status_content = content.item_status();
                        let status_thumbnail = thumbnail.item_status();

                        let status_short_snslink = sns_links.item_status_has_bad();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_basic_info}
{status_authors}
{status_duration}
{status_content}
{status_thumbnail}

他のページの内容まとめ:
SNSリンク: {status_short_snslink}
                        "#
                        );

                        let button_basic_info = basic_info.create_discord_button();
                        let button_authors = authors.create_discord_button();
                        let button_duration = duration.create_discord_button();
                        let button_content = content.create_discord_button();
                        let button_thumbnail = thumbnail.create_discord_button();

                        vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                            CreateComponent::ActionRow(CreateActionRow::Buttons(Cow::Owned(vec![
                                button_basic_info,
                                button_authors,
                                button_duration,
                                button_content,
                                button_thumbnail,
                            ]))),
                        ]
                    }
                    OtherWorkStatePage::SnsLink => {
                        let status_snslink = sns_links.items_statuses();

                        let status_text_display = format!(
                            r#"
{file_name_msg}
{DISCORD_MESSAGE_FRONTMATTER}

{DISCORD_MESSAGE_EMOJI_STATUS_EXPLATINATION}

{status_snslink}
                        "#
                        );

                        let mut status_components = vec![
                            CreateComponent::TextDisplay(CreateTextDisplay::new(Cow::Owned(
                                status_text_display,
                            ))),
                            CreateComponent::Separator(CreateSeparator::new()),
                        ];
                        status_components.append(&mut sns_links.create_button_list());
                        status_components
                    }
                }
            }
        };

        components.push(CreateComponent::Separator(CreateSeparator::new()));
        components.push(CreateComponent::ActionRow(CreateActionRow::Buttons(
            Cow::Owned(confirm_cancel_turnpage_buttons),
        )));

        components
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModalWrapper<T>
where
    T: FromModal,
{
    Unset,
    InitWithValue(T),
    Ok { input: T::Input, value: T },
    Bad { input: T::Input, error: String },
}

impl<T> ModalWrapper<T>
where
    T: FromModal,
{
    pub const fn id() -> ComponentInteractionTypes {
        ComponentInteractionTypes::Singular(T::ID)
    }

    pub const fn required() -> Required {
        T::REQUIRED
    }

    pub fn create_discord_button(&self) -> CreateButton<'_> {
        let status = match self {
            ModalWrapper::Unset => match Self::required() {
                Required::Yes => ButtonStyle::Danger,
                Required::Recommended => ButtonStyle::Primary,
                Required::Optional => ButtonStyle::Secondary,
            },
            ModalWrapper::Ok { .. } => ButtonStyle::Success,
            ModalWrapper::Bad { .. } => ButtonStyle::Danger,
            ModalWrapper::InitWithValue(_) => ButtonStyle::Primary,
        };
        Self::id().to_button().style(status)
    }

    pub fn item_status(&self) -> String {
        let status = match &self {
            ModalWrapper::Unset => "未定義値".to_string(),
            ModalWrapper::Ok { .. } => "良好".to_string(),
            ModalWrapper::Bad { error, .. } => format!("エラー: {:?}", error),
            ModalWrapper::InitWithValue(_) => "基本値".to_string(),
        };

        format!(
            r#"
- {}: {}
            "#,
            Self::id().to_button_label(),
            status
        )
    }

    pub fn is_ok(&self) -> bool {
        match self {
            ModalWrapper::Unset => match Self::required() {
                Required::Yes => false,
                Required::Recommended | Required::Optional => true,
            },
            ModalWrapper::Ok { .. } => true,
            ModalWrapper::Bad { .. } => false,
            ModalWrapper::InitWithValue(_) => true,
        }
    }
}

impl<T> Default for ModalWrapper<T>
where
    T: FromModal,
{
    fn default() -> Self {
        ModalWrapper::Unset
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModalListWrapperIndividualItem<T>
where
    T: FromListModal + Serialize + DeserializeOwned,
{
    InitWithValue(T),
    Ok {
        input: T::AddOrIndividualInput,
        value: T,
    },
    Bad {
        input: T::AddOrIndividualInput,
        error: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModalListWrapper<T>
where
    T: FromListModal + Debug,
{
    items: Vec<ModalListWrapperIndividualItem<T>>,
}

impl<T> ModalListWrapper<T>
where
    T: FromListModal + Serialize + DeserializeOwned,
{
    pub const fn add_id() -> ComponentInteractionTypes {
        ComponentInteractionTypes::Multiple(MultipleEditComponents {
            interact_type: T::MULTI_TYPE,
            action_type: ListInteraction::Add,
        })
    }
    pub const fn remove_id() -> ComponentInteractionTypes {
        ComponentInteractionTypes::Multiple(MultipleEditComponents {
            interact_type: T::MULTI_TYPE,
            action_type: ListInteraction::Remove,
        })
    }
    pub const fn item_id(idx: u8) -> ComponentInteractionTypes {
        ComponentInteractionTypes::Multiple(MultipleEditComponents {
            interact_type: T::MULTI_TYPE,
            action_type: ListInteraction::Edit(idx),
        })
    }
    pub fn required() -> Required {
        T::REQUIRED
    }

    pub fn new(values: Vec<T>) -> Result<Self, Error> {
        if values.len() > 25 {
            return Err(Error::msg("Contract Violation - Too many values!"));
        }
        Ok(Self {
            items: values
                .into_iter()
                .map(|x| ModalListWrapperIndividualItem::InitWithValue(x))
                .collect(),
        })
    }

    pub fn create_button_list(&self) -> Vec<CreateComponent<'_>> {
        let items_iter = self.items.iter();
        let all_buttons = items_iter
            .enumerate()
            .map(|(index, item)| {
                let status = match item {
                    ModalListWrapperIndividualItem::InitWithValue(_) => match Self::required() {
                        Required::Yes => ButtonStyle::Primary,
                        Required::Recommended => ButtonStyle::Primary,
                        Required::Optional => ButtonStyle::Secondary,
                    },
                    ModalListWrapperIndividualItem::Ok { .. } => ButtonStyle::Success,
                    ModalListWrapperIndividualItem::Bad { .. } => ButtonStyle::Danger,
                };
                Self::item_id(index as u8).to_button().style(status)
            })
            .collect::<Vec<CreateButton<'_>>>();

        // discord limits us to 5 buttons per row. due to this
        // we split the buttons into multiple rows of buttons
        let mut list_of_multiple_buttons = all_buttons
            .chunks(5)
            .map(|chubks| {
                CreateComponent::ActionRow(CreateActionRow::Buttons(Cow::Owned(chubks.to_vec())))
            })
            .collect::<Vec<CreateComponent<'_>>>();

        // _shakes fist_ if only create button fns were const...
        let default_buttons = CreateActionRow::Buttons(Cow::Owned(vec![
            Self::add_id().to_button(),
            Self::remove_id().to_button(),
        ]));

        list_of_multiple_buttons.push(CreateComponent::ActionRow(default_buttons));
        list_of_multiple_buttons
    }

    pub fn items_statuses(&self) -> String {
        if self.items.is_empty() {
            return "現在、入力がありません。".to_string();
        }

        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let status = match item {
                    ModalListWrapperIndividualItem::InitWithValue(_) => Cow::Borrowed("基本値"),
                    ModalListWrapperIndividualItem::Ok { .. } => Cow::Borrowed("良好"),
                    ModalListWrapperIndividualItem::Bad { input: _, error } => {
                        Cow::Owned(format!("エラー発生: {error}"))
                    }
                };

                format!("- #{}: {}\n", index + 1, status)
            })
            .collect::<String>()
    }

    pub fn item_status_has_bad(&self) -> &'static str {
        match self.items.iter().any(|item| match item {
            ModalListWrapperIndividualItem::InitWithValue(_)
            | ModalListWrapperIndividualItem::Ok { .. } => false,
            ModalListWrapperIndividualItem::Bad { .. } => true,
        }) {
            true => "良好",
            false => "エラーあり！！！",
        }
    }
}

impl<T> Default for ModalListWrapper<T>
where
    T: FromListModal,
{
    fn default() -> Self {
        Self { items: vec![] }
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

trait FromModal: Sized + Clone + Debug + Serialize + DeserializeOwned {
    const ID: SingularEditComponents;
    const REQUIRED: Required;

    type Input: Modal;

    fn parse(context: &Work, input: Self::Input) -> Result<Self, Error>;

    fn to_input(&self) -> Self::Input;
}

trait FromListModal: Sized + Clone + Debug + Serialize + DeserializeOwned {
    const MULTI_TYPE: MultiInteractType;
    const REQUIRED: Required;

    type AddOrIndividualInput: Modal + Clone + Debug + Serialize + DeserializeOwned;
    type DeleteInput: Modal + Clone + Debug + Serialize + DeserializeOwned;

    fn parse_individual(context: &Work, input: Self::AddOrIndividualInput) -> Result<Self, Error>;

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error>;
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BasicInfo {
    pub title: String,
    pub short_description: String,
    pub date: NaiveDate,
}

impl FromModal for BasicInfo {
    const ID: SingularEditComponents = SingularEditComponents::BasicInfo;
    const REQUIRED: Required = Required::Yes;

    type Input = BasicInfoModal;

    fn parse(_context: &Work, input: Self::Input) -> Result<Self, Error> {
        let date = NaiveDate::parse_from_str(&input.date, "%Y-%m-%d")?;
        Ok(BasicInfo {
            title: input.title.into_string(),
            short_description: input.short.into_string(),
            date,
        })
    }

    fn to_input(&self) -> Self::Input {
        BasicInfoModal {
            title: FixedString::from_str_trunc(&self.title),
            short: FixedString::from_str_trunc(&self.short_description),
            date: FixedString::from_string_trunc(self.date.to_string()),
        }
    }
}

fn parse_authors(name_map: &NameMap, authors_str: &str) -> Result<Vec<String>, Error> {
    authors_str
        .split(",")
        .map(str::trim)
        .map(|x| match x.chars().all(|c| c.is_ascii_alphanumeric()) {
            true => match name_map.members.get(x) {
                Some(_) => Ok(x.to_string()),
                None => Err(Error::msg("author parsing failed: author doesn't exist")),
            },
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

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Authors {
    pub authors: Vec<String>,
    pub additional_authors: Vec<String>,
}

impl FromModal for Authors {
    const ID: SingularEditComponents = SingularEditComponents::Authors;
    const REQUIRED: Required = Required::Yes;

    type Input = AuthorsModal;

    fn parse(context: &Work, input: Self::Input) -> Result<Self, Error> {
        let authors = parse_authors(&context.name_map, &input.authors)?;
        let additional_authors = parse_additional_authors(&input.additional_authors);

        Ok(Authors {
            authors,
            additional_authors,
        })
    }

    fn to_input(&self) -> Self::Input {
        AuthorsModal {
            authors: FixedString::from_string_trunc(self.authors.join(", ").to_string()),
            additional_authors: FixedString::from_string_trunc(
                self.additional_authors.join(", ").to_string(),
            ),
        }
    }
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
#[name = "ページ内容"]
#[text_display = "マニュアル:　"]
pub struct PageContentModal {
    #[name = "（ページの内容物（例:　曲の歌詞）"]
    pub content: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PageContent {
    pub content: String,
}

impl FromModal for PageContent {
    const ID: SingularEditComponents = SingularEditComponents::Content;
    const REQUIRED: Required = Required::Recommended;

    type Input = PageContentModal;

    fn parse(_context: &Work, input: Self::Input) -> Result<Self, Error> {
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

    fn to_input(&self) -> Self::Input {
        PageContentModal {
            content: FixedString::from_str_trunc(&self.content),
        }
    }
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
#[name = "主要リンク"]
#[text_display = "マニュアル:　"]
pub struct MainLinkModal {
    #[name = "主要リンク先"]
    pub link: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MainLink {
    pub url: Url,
}

impl FromModal for MainLink {
    const ID: SingularEditComponents = SingularEditComponents::Link;
    const REQUIRED: Required = Required::Recommended;

    type Input = MainLinkModal;

    fn parse(_context: &Work, input: Self::Input) -> Result<Self, Error> {
        Ok(MainLink {
            url: Url::parse(&input.link)?,
        })
    }

    fn to_input(&self) -> Self::Input {
        MainLinkModal {
            link: FixedString::from_str_trunc(self.url.as_str()),
        }
    }
}

#[derive(Clone, Debug, Modal, Serialize, Deserialize)]
#[name = "サムネイル"]
#[text_display = "マニュアル:　"]
pub struct ThumbnailModal {
    #[name = "イメージファイル名（まずアップロード必須)"]
    #[max_length = 50]
    #[min_length = 1]
    pub image: FixedString<u16>,
    #[name = "メンバーページある作成者"]
    #[description = "名は\",\"で分離して入力してください。英語字幕のみです。（例: mitsumori, knoeze, seeyoumayday)"]
    pub authors: FixedString<u16>,
    #[name = "メンバーページない作成者"]
    #[description = "名は\",\"で分離して入力してください。（例:　リリイ・シュシュ, 金魚光線, Maurice Ravel）"]
    pub additional_authors: FixedString<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub illustration: Illustration,
}

impl FromModal for Thumbnail {
    const ID: SingularEditComponents = SingularEditComponents::Thumbnail;
    const REQUIRED: Required = Required::Optional;

    type Input = ThumbnailModal;

    fn parse(context: &Work, input: Self::Input) -> Result<Self, Error> {
        check_file_exists(
            &context.directory,
            PathSection::Image,
            &input.image.as_str(),
        )?;

        let illustrators = parse_authors(&context.name_map, &input.authors)?;
        let additional_illustrators = parse_additional_authors(&input.additional_authors);

        let illustration = Illustration {
            image: input.image.to_string(),
            illustrators,
            additional_illustrators,
            description: Some("サムネイル".to_string()),
        };

        Ok(Thumbnail { illustration })
    }

    fn to_input(&self) -> Self::Input {
        ThumbnailModal {
            image: FixedString::from_str_trunc(&self.illustration.image),
            authors: FixedString::from_string_trunc(
                self.illustration.illustrators.join(", ").to_string(),
            ),
            additional_authors: FixedString::from_string_trunc(
                self.illustration
                    .additional_illustrators
                    .join(", ")
                    .to_string(),
            ),
        }
    }
}

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
#[name = "曲の長さ入力"]
pub struct DurationModal {
    #[max_length = 5]
    #[min_length = 1]
    #[name = "（必須）アルバムトラック長さ"]
    #[description = "MMm SSs　（例:　３分２５秒は 3m 25s)"]
    pub length_mmss: FixedString<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackDuration(pub FancyDuration<Duration>);

impl FromModal for TrackDuration {
    const ID: SingularEditComponents = SingularEditComponents::Duration;
    const REQUIRED: Required = Required::Yes;

    type Input = DurationModal;

    fn parse(_context: &Work, input: Self::Input) -> Result<Self, Error> {
        Ok(TrackDuration(FancyDuration::parse(&input.length_mmss)?))
    }

    fn to_input(&self) -> Self::Input {
        DurationModal {
            length_mmss: FixedString::from_string_trunc(self.0.to_string()),
        }
    }
}

#[derive(Clone, Debug, Modal, Serialize, Deserialize)]
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
    #[name = "プロファイル写真（まずアップロード必須)"]
    #[max_length = 50]
    #[min_length = 1]
    pub profile_picture: FixedString<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemberBasicInfo {
    pub name: String,
    pub ascii_name: String,
    pub short: String,
    pub profile_picture: String,
}

impl FromModal for MemberBasicInfo {
    const ID: SingularEditComponents = SingularEditComponents::ArtistInfo;
    const REQUIRED: Required = Required::Yes;

    type Input = MemberBasicInfoModal;

    fn parse(context: &Work, input: Self::Input) -> Result<Self, Error> {
        input
            .ascii_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
            .ok_or(Error::msg("Illegal character in `ascii_name`"))?;

        if input.name.contains(",") || input.name.contains("、") {
            return Err(Error::msg("Illegal character in `name`"));
        }

        if let Some(_) = context.name_map.members.get(input.ascii_name.as_str()) {
            return Err(Error::msg(
                "Name conflict: ascii name already exists in name map!",
            ));
        }

        if let Some(_) = context
            .name_map
            .members
            .values()
            .find(|item| item.as_str() == input.name.as_str())
        {
            return Err(Error::msg(
                "Name conflict: name already exists in name map!",
            ));
        }

        check_file_exists(
            &context.directory,
            PathSection::Image,
            &input.profile_picture.as_str(),
        )?;

        Ok(MemberBasicInfo {
            name: input.name.into_string(),
            ascii_name: input.ascii_name.into_string(),
            short: input.short.into_string(),
            profile_picture: input.profile_picture.into_string(),
        })
    }

    fn to_input(&self) -> Self::Input {
        MemberBasicInfoModal {
            name: FixedString::from_str_trunc(&self.name),
            ascii_name: FixedString::from_str_trunc(&self.ascii_name),
            short: FixedString::from_str_trunc(&self.short),
            profile_picture: FixedString::from_str_trunc(&self.short),
        }
    }
}

#[derive(Clone, Debug, Modal, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ExtendedMemberBasicInfo {
    pub department: Option<String>,
    pub position: Option<String>,
    pub entry: Option<String>,
}

impl FromModal for ExtendedMemberBasicInfo {
    const ID: SingularEditComponents = SingularEditComponents::AdditionlArtistInfo;
    const REQUIRED: Required = Required::Optional;

    type Input = ExtendedMemberBasicInfoModal;

    fn parse(_context: &Work, input: Self::Input) -> Result<Self, Error> {
        Ok(ExtendedMemberBasicInfo {
            department: input.department.map(FixedString::into_string),
            position: input.position.map(FixedString::into_string),
            entry: input.entry.map(FixedString::into_string),
        })
    }

    fn to_input(&self) -> Self::Input {
        ExtendedMemberBasicInfoModal {
            department: self
                .department
                .as_ref()
                .map(|n| FixedString::from_str_trunc(n)),
            position: self
                .position
                .as_ref()
                .map(|n| FixedString::from_str_trunc(n)),
            entry: self.entry.as_ref().map(|n| FixedString::from_str_trunc(n)),
        }
    }
}

// from here, modals that can have multiples (sns link, track, illusts)
//

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
#[name = "SNSリンク追加"]
#[text_display = "マニュアル:　"]
pub struct SnsLinksModal {
    pub link: FixedString<u16>,
}

// from here, modals that can have multiples (sns link, track, illusts)
#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
#[name = "SNSリンク消去"]
#[text_display = "マニュアル:　"]
pub struct DeleteSnsLinksModal {
    #[name = "（必須）消去するリンクの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SnsLink {
    pub url: Url,
}

impl FromListModal for SnsLink {
    const MULTI_TYPE: MultiInteractType = MultiInteractType::SnsLink;

    const REQUIRED: Required = Required::Optional;

    type AddOrIndividualInput = SnsLinksModal;
    type DeleteInput = DeleteSnsLinksModal;

    fn parse_individual(_context: &Work, input: Self::AddOrIndividualInput) -> Result<Self, Error> {
        Ok(SnsLink {
            url: Url::parse(&input.link)?,
        })
    }

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error> {
        Ok(input.number.parse::<usize>()?)
    }
}

#[derive(Clone, Debug, Modal, Serialize, Deserialize)]
#[name = "イラスト追加"]
pub struct IllustrationModal {
    #[name = "このイラストの題名"]
    #[max_length = 50]
    #[min_length = 1]
    pub title: FixedString<u16>,
    #[name = "このイラストの説明"]
    #[max_length = 100]
    pub description: FixedString<u16>,
    #[name = "イメージファイル名（まずアップロード必須)"]
    #[max_length = 50]
    #[min_length = 1]
    pub image: FixedString<u16>,
    #[name = "メンバーページある作成者"]
    #[description = "名は\",\"で分離して入力してください。英語字幕のみです。（例: mitsumori, knoeze, seeyoumayday)"]
    pub authors: FixedString<u16>,
    #[name = "メンバーページない作成者"]
    #[description = "名は\",\"で分離して入力してください。（例:　リリイ・シュシュ, 金魚光線, Maurice Ravel）"]
    pub additional_authors: FixedString<u16>,
}

#[derive(Clone, Debug, Modal, Serialize, Deserialize)]
#[name = "イラスト消去"]
#[text_display = "この編集から追加したイラストを消す"]
pub struct DeleteIllustrationModal {
    #[name = "（必須）消去するイラストの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TitleAndIllustraion {
    pub title: String,
    pub illustration: Illustration,
}

impl FromListModal for TitleAndIllustraion {
    const REQUIRED: Required = Required::Optional;

    type AddOrIndividualInput = IllustrationModal;

    type DeleteInput = DeleteIllustrationModal;

    fn parse_individual(context: &Work, input: Self::AddOrIndividualInput) -> Result<Self, Error> {
        let illustrators = parse_authors(&context.name_map, &input.authors)?;
        let additional_illustrators = parse_additional_authors(&input.additional_authors);

        check_file_exists(&context.directory, PathSection::Image, &input.image);

        let illustration = Illustration {
            image: input.image.into_string(),
            illustrators,
            additional_illustrators,
            description: Some(input.description.into_string()),
        };

        Ok(TitleAndIllustraion {
            title: input.title.into_string(),
            illustration,
        })
    }

    fn parse_delete(input: Self::DeleteInput) -> Result<usize, Error> {
        Ok(input.number.parse::<usize>()?)
    }

    const MULTI_TYPE: MultiInteractType = MultiInteractType::Illustrations;
}

// album tracks

#[derive(Clone, Debug, Modal, PartialEq, PartialOrd, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Modal, Serialize, Deserialize)]
#[name = "トラック消去"]
#[text_display = "この編集から追加したトラックを消す"]
pub struct DeleteTrackModal {
    #[name = "（必須）消去するトラックの番号"]
    #[max_length = 2]
    pub number: FixedString<u16>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AlbumTitleAndTrack {
    pub title: String,
    pub track: Track,
}

impl FromListModal for AlbumTitleAndTrack {
    const MULTI_TYPE: MultiInteractType = MultiInteractType::AlbumTracks;
    const REQUIRED: Required = Required::Optional;

    type AddOrIndividualInput = AlbumTrackModal;
    type DeleteInput = DeleteTrackModal;

    fn parse_individual(context: &Work, input: Self::AddOrIndividualInput) -> Result<Self, Error> {
        let title = input.title.to_string();
        let duration = FancyDuration::parse(&input.length_mmss)?;
        let authors = parse_authors(&context.name_map, &input.authors)?;
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

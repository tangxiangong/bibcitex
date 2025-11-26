use bibcitex_core::bib::Reference;
use biblatex::EntryType;
use dioxus::prelude::*;

mod article;
pub use article::*;
mod booklet;
pub use booklet::*;
mod book;
pub use book::*;
mod inbook;
pub use inbook::*;
mod incollection;
pub use incollection::*;
mod inproceeding;
pub use inproceeding::*;
mod misc;
pub use misc::*;
mod search;
pub use search::*;
mod selector;
pub use selector::*;
mod techreport;
pub use techreport::*;
mod thesis;
pub use thesis::*;
mod unimplemented;
pub use unimplemented::*;

#[component]
pub fn HelperComponent(entry: Reference) -> Element {
    rsx! {
        match entry.type_ {
            EntryType::Article => rsx! {
                ArticleHelper { entry }
            },
            EntryType::Book => rsx! {
                BookHelper { entry }
            },
            EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => {
                rsx! {
                    ThesisHelper { entry }
                }
            }
            EntryType::InProceedings => rsx! {
                InProceedingsHelper { entry }
            },
            EntryType::TechReport => rsx! {
                TechReportHelper { entry }
            },
            EntryType::Misc => rsx! {
                MiscHelper { entry }
            },
            EntryType::Booklet => rsx! {
                BookletHelper { entry }
            },
            EntryType::InBook => rsx! {
                InBookHelper { entry }
            },
            EntryType::InCollection => rsx! {
                InCollectionHelper { entry }
            },
            _ => rsx! {
                UnimplementedHelper { entry }
            },
        }
    }
}

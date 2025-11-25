use bibcitex_core::bib::Reference;
use biblatex::EntryType;
use dioxus::prelude::*;

mod article;
pub use article::*;
mod booklet;
pub use booklet::*;
mod book;
pub use book::*;
mod chunks_comp;
pub use chunks_comp::*;
mod inbook;
pub use inbook::*;
mod incollection;
pub use incollection::*;
mod inproceedings;
pub use inproceedings::*;
mod misc;
pub use misc::*;
mod selector;
pub use selector::*;
mod techreport;
pub use techreport::*;
mod thesis;
pub use thesis::*;
mod unimplemented;
pub use unimplemented::*;

#[component]
pub fn ReferenceComponent(entry: Reference) -> Element {
    rsx! {
        match entry.type_ {
            EntryType::Article => rsx! {
                Article { entry }
            },
            EntryType::Book => rsx! {
                Book { entry }
            },
            EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => {
                rsx! {
                    Thesis { entry }
                }
            }
            EntryType::InProceedings => rsx! {
                InProceedings { entry }
            },
            EntryType::TechReport => rsx! {
                TechReport { entry }
            },
            EntryType::Misc => rsx! {
                Misc { entry }
            },
            EntryType::Booklet => rsx! {
                Booklet { entry }
            },
            EntryType::InBook => rsx! {
                InBook { entry }
            },
            EntryType::InCollection => rsx! {
                InCollection { entry }
            },
            _ => rsx! {
                Unimplemented { entry }
            },
        }
    }
}

#[component]
pub fn ReferenceDrawer(entry: Reference) -> Element {
    match entry.type_.clone() {
        EntryType::Article => rsx! {
            ArticleDrawer { entry }
        },
        EntryType::Book => rsx! {
            BookDrawer { entry }
        },
        EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => rsx! {
            ThesisDrawer { entry }
        },
        EntryType::InProceedings => rsx! {
            InProceedingsDrawer { entry }
        },
        EntryType::TechReport => rsx! {
            TechReportDrawer { entry }
        },
        EntryType::Misc => rsx! {
            MiscDrawer { entry }
        },
        EntryType::Booklet => rsx! {
            BookletDrawer { entry }
        },
        EntryType::InBook => rsx! {
            InBookDrawer { entry }
        },
        EntryType::InCollection => rsx! {
            InCollectionDrawer { entry }
        },
        _ => rsx! {
            UnimplementedDrawer { entry }
        },
    }
}

use freya::prelude::*;

use crate::core::rendering::LineElement;

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(lines: SyncSignal<Vec<LineElement>>) -> Element {
    rsx!(
        rect {
            padding: "50 50 20 100",
            for line in lines() {
                rect {
                    padding: "2 0",
                    paragraph {
                        for segment in line.segments() {
                            text { "{segment.text}" }
                        }
                    }
                }
            }
        }
    )
}

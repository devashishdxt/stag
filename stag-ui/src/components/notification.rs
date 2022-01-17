use bounce::use_atom;
use yew::prelude::*;

use crate::atoms::NotificationAtom;

#[function_component(Notification)]
pub fn notification() -> Html {
    let atom = use_atom::<NotificationAtom>();

    match (*atom).data {
        None => html! {
            <></>
        },
        Some(ref data) => html! {
            <div class="notification-container">
                <div>{ data.message.clone() }
                {
                    if data.dismissable {
                        html! {
                            <button onclick={ move |_| {
                                atom.set(NotificationAtom { data: None });
                            }}>{ "Dismiss" }</button>
                        }
                    } else {
                        html! { <></> }
                    }
                }
                </div>
            </div>
        },
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::slint! {
    #[edition = "1.19"]

    import { Button, VerticalBox } from "std-widgets.slint";

    export component AppWindow inherits Window {
        title: "Ticky";
        preferred-width: 460px;
        preferred-height: 280px;
        background: #0e0f12;

        in-out property <string> status: "Ticky is running";

        VerticalBox {
            alignment: center;
            padding: 24px;
            spacing: 16px;

            Text {
                text: "Ticky";
                color: #e8eaed;
                font-size: 28px;
                font-weight: 700;
                horizontal-alignment: center;
            }

            Text {
                text: root.status;
                color: #9aa0aa;
                font-size: 14px;
                horizontal-alignment: center;
            }

            Button {
                text: "Test";
                clicked => {
                    root.status = "UI is alive - audio comes next";
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    window.run()
}

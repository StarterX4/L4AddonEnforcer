use fltk::{button::*, input::Input, output::Output, enums::*, valuator::*, prelude::*, utils::oncelock::Lazy, *};
//use group::Scroll;
use std::ops::{Deref, DerefMut};
pub use fltk_theme::widget_schemes::sweet::frames::*;
pub use fltk_theme::{SchemeType, WidgetScheme};
//use fltk_sys::

pub use fltk_theme::colors::sweet::dark::*; // get all the dark sweet colors

// use fltk_theme::colors::sweet::light::*; // get all the light aqua colors

// use fltk_theme::colors::sweet::sys::*; // get all the system aqua colors, requires MacOS

pub fn apply_theme() {
    app::set_font_size(14);
    let bg = windowBackgroundColor.to_rgb();
    app::background(bg.0, bg.1, bg.2);
    let ctrl = controlAccentColor.to_rgb(); //Sweet's original green color for checkboxes
    //let ctrl = selectedControlColor.to_rgb(); //purple
    app::background2(ctrl.0, ctrl.1, ctrl.2);
    let lbl = labelColor.to_rgb();
    app::foreground(lbl.0, lbl.1, lbl.2);
    let txt_sel_bg = selectedTextBackgroundColor.to_rgb();
    app::set_selection_color(txt_sel_bg.0, txt_sel_bg.1, txt_sel_bg.2);
    //app::set_color(Color::Selection, 255, 255, 255);
    let widget_scheme = WidgetScheme::new(SchemeType::Aqua);
    widget_scheme.apply();
}

// Buttons
pub struct RButton {
    b: Button,
}

fn RButton_common(b: &mut Button) {
        b.set_label_size(14);
        b.set_compact(true);
        b.handle(move |b, ev| match ev {
            Event::Enter => {
                //b.set_frame(OS_DEFAULT_HOVERED_UP_BOX);
                b.set_color(*selectedControlColor);
                b.redraw();
                true
            }
            Event::Leave => {
                //b.set_frame(OS_DEFAULT_BUTTON_UP_BOX);
                b.set_color(*controlColor);
                b.redraw();
                true
            }
            _ => false,
        });
        b.set_selection_color(*selectedControlColor);
        b.set_frame(OS_DEFAULT_BUTTON_UP_BOX);
        b.set_color(*controlColor);
}

impl RButton {
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> RButton {
        let mut b = button::Button::new(x, y, w, h, title);
        RButton_common(&mut b);
        Self { b }
    }
    pub fn default(title: &str) -> RButton {
        let mut b = button::Button::default().with_label(title);
        RButton_common(&mut b);
        Self { b }
    }
}

impl Deref for RButton {
    type Target = Button;

    fn deref(&self) -> &Self::Target {
        &self.b
    }
}

impl DerefMut for RButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.b
    }
}

// TextInput
pub struct RInput {
    inp: Input,
}

fn Rinput_common(inp: &mut Input) {
        inp.set_color(*controlColor);
        inp.set_selection_color(*selectedControlColor);
        inp.set_text_color(*labelColor);
        inp.set_cursor_color(*controlAccentColor);
}

impl RInput {
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> RInput {
        let mut inp = input::Input::new(x, y, w, h, "");
        Rinput_common(&mut inp);
        Self { inp }
    }
    pub fn default(title: &str) -> RInput {
        let mut inp = input::Input::default().with_label(title);
        Rinput_common(&mut inp);
        Self { inp }
    }
}

impl Clone for RInput {
    fn clone(&self) -> RInput {
        // Create a new instance of `RInput` with the same properties
        RInput::new(self.inp.x(), self.inp.y(), self.inp.width(), self.inp.height(), &self.inp.label())
    }
}

impl Deref for RInput {
    type Target = Input;
    fn deref(&self) -> &Self::Target {
        &self.inp
    }
}
    impl DerefMut for RInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inp
    }
}

// TextOutput
pub struct ROutput {
    out: Output,
}

fn ROutput_common(out: &mut Output) {
    out.set_color(*controlColor);
    out.set_selection_color(*selectedControlColor);
    out.set_text_color(*labelColor);
    out.set_cursor_color(*controlAccentColor);
}

impl ROutput {
    pub fn new(x: i32, y: i32, w: i32, h: i32, title: &str) -> ROutput {
        let mut out = output::Output::new(x, y, w, h, "");
        ROutput_common(&mut out);
        Self { out }
    }
    pub fn default(title: &str) -> ROutput {
        let mut out = output::Output::default().with_label(title);
        ROutput_common(&mut out);
        Self { out }
    }
}

impl Deref for ROutput {
    type Target = Output;
    fn deref(&self) -> &Self::Target {
        &self.out
    }
}
    impl DerefMut for ROutput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.out
    }
}

// Scrollbar — doesn't work just for scrollbar, 
// but modifies the whole group area, except the scrollbar ofc (wtf)

// pub struct RScrollbar {
//     sb: Scrollbar,
// }

// fn RScrollbar_common(sb: &mut Scrollbar) {
//     static inactive_color: Lazy<Color> = Lazy::new(|| Color::from_rgba_tuple((66, 75, 112, 128))); // Semi-transparent version
//     let active_color = Color::from_rgb(66, 75, 112);

//     sb.set_color(*inactive_color); // Initial color (semi-transparent)

//     sb.handle(move |s, ev| match ev {
//         Event::Enter => {
//             s.set_color(active_color);
//             s.redraw();
//             true
//         }
//         Event::Leave => {
//             s.set_color(*inactive_color);
//             s.redraw();
//             true
//         }
//         _ => false,
//     });
// }

// impl RScrollbar { // also for as a group::scroll replacement [?]
//     pub fn new(x: i32, y: i32, w: i32, h: i32) -> RScrollbar {
//         let mut sb = Scrollbar::new(x, y, w, h, "");
//         RScrollbar_common(&mut sb);
//         Self { sb }
//     }
//     pub fn default() -> RScrollbar {
//         let mut sb = Scrollbar::default();
//         RScrollbar_common(&mut sb);
//         Self { sb }
//     }
// }

// impl Deref for RScrollbar {
//     type Target = Scrollbar;
//     fn deref(&self) -> &Self::Target {
//         &self.sb
//     }
// }
//     impl DerefMut for RScrollbar {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.sb
//     }
// }


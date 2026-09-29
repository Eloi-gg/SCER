use cgi::factory_widgets as fw;
use cgi::widget::WidgetBuilder;
use cgi::{Layout, Widget, WidgetPlacement};

pub struct ScerUi {
    next_instruction: modules::NextInstruction,
    registers: modules::Registers,
    screen: modules::Screen,
    message_boards: modules::MessageBoards,
}

impl ScerUi {
    pub fn new() -> Self {
        Self {
            next_instruction: modules::NextInstruction::new(),
            registers: modules::Registers::new(),
            screen: modules::Screen::new(),
            message_boards: modules::MessageBoards::new(),
        }
    }

    pub fn add_all_layouts(&self, app: &mut cgi::application::Application) {
        app.add_layout(0, self.small());

        app.set_layout_behaviour(|size| Self::update(size));
    }

    fn update(size: (u16, u16)) -> u8 {
        scer_layouts::SMALL_LAYOUT
    }

    fn small(&self) -> Layout {
        use sizes::*;

        // Placing
        let screen_placement = WidgetPlacement::new_with_size(0, 0, SCREEN_SIZE.0, SCREEN_SIZE.1);
        let next_instruction_placement = WidgetPlacement::new_with_size(
            0,
            SCREEN_SIZE.1 + 1,
            NEXT_INSTRUCTION_SIZE.0,
            NEXT_INSTRUCTION_SIZE.1,
        );
        let registers_placement = WidgetPlacement::new_with_size(
            0,
            SCREEN_SIZE.1 + NEXT_INSTRUCTION_SIZE.1 + 2,
            REGISTERS_SIZE.0,
            REGISTERS_SIZE.1,
        );
        let new_messages_board_placement = WidgetPlacement::new_with_size(
            0,
            SCREEN_SIZE.1 + NEXT_INSTRUCTION_SIZE.1 + REGISTERS_SIZE.1 + 3,
            NEW_MESSAGES_SIZE.0,
            NEW_MESSAGES_SIZE.1,
        );
        let old_messages_board_placement = new_messages_board_placement
            .get_below()
            .with_bottom_right_y(1.0);

        // Widgets
        let mut layout = Layout::new();
        layout.add_widget(&self.screen.0, screen_placement);
        layout.add_widget(&self.next_instruction.widget, next_instruction_placement);
        self.registers
            .add_to_layout(registers_placement, &mut layout);
        self.message_boards.add_to_layout(
            old_messages_board_placement,
            new_messages_board_placement,
            &mut layout,
        );

        layout
    }
}

mod scer_layouts {
    use super::*;

    pub(super) const SMALL_LAYOUT: u8 = 0;
}

mod sizes {
    pub(super) const SCREEN_SIZE: (i32, i32) = (16, 4);
    pub(super) const NEXT_INSTRUCTION_SIZE: (i32, i32) = (38, 4);
    pub(super) const REGISTERS_SIZE: (i32, i32) = (38, 6);
    pub(super) const NEW_MESSAGES_SIZE: (i32, i32) = (38, 5);
}

mod modules {
    use super::*;
    use cgi::factory_widgets::text::Wrapping;

    type TbWidget = Widget<fw::text::TextBox>;

    pub(super) struct Screen(pub Widget<fw::utils::Empty>);
    pub(super) struct NextInstruction {
        pub widget: TbWidget,
        last_instruction: u32,
    }
    pub(super) struct Registers {
        pub widgets: Vec<TbWidget>,
        outline: Widget<fw::utils::Empty>,
        registers: [u16; Self::NUM_REGISTERS + Self::NUM_SPECIAL_REGISTERS],
    }
    pub(super) struct MessageBoards {
        old_messages: TbWidget,
        new_messages: TbWidget,
    }

    mod msg_board {
        use super::*;

        pub(super) fn new(title: &str) -> TbWidget {
            let text_box =
                fw::text::TextBox::new("", fw::Listener::empty(), fw::text::TextAlign::Left)
                    .with_wrapping_mode(fw::text::Wrapping::Off);
            WidgetBuilder::new(text_box)
                .with_outline(cgi::symbols::OutlineStyle::Rounded)
                .with_title(title)
                .build()
        }

        pub(super) fn add_message(widget: &mut TbWidget, message: &str) {
            let mut edit = widget.edit();
            let old_text = edit.text();
            let next_text = message.to_owned() + "\n" + &old_text;
            edit.set_text(&next_text);
        }
    }

    impl Screen {
        pub(super) fn new() -> Self {
            Self(
                WidgetBuilder::new(fw::utils::Empty)
                    .with_outline(cgi::symbols::OutlineStyle::Double)
                    .build(),
            )
        }
    }
    impl NextInstruction {
        pub(super) fn new() -> Self {
            let empty_listener = fw::Listener::empty();
            let widget = WidgetBuilder::new(
                fw::text::TextBox::new(
                    "not set yet",
                    empty_listener.clone(),
                    fw::text::TextAlign::Left,
                )
                .with_wrapping_mode(Wrapping::Off),
            )
            .with_outline(cgi::symbols::OutlineStyle::Normal)
            .with_title("Next instruction")
            .build();

            Self {
                widget,
                last_instruction: 0,
            }
        }

        pub(super) fn set(&mut self, instruction: u32) {
            let decoded_instruction = crate::program::Instruction::from_binary(instruction);
            self.widget //TODO: we dont need all 32 bits!
                .edit()
                .set_text(&format!("{:#034b}\n{:?}", instruction, decoded_instruction));
        }
    }

    impl Registers {
        const NUM_REGISTERS: usize = 8;
        const NUM_SPECIAL_REGISTERS: usize = 2;

        pub fn new() -> Self {
            let total_n_reg = Self::NUM_REGISTERS + Self::NUM_SPECIAL_REGISTERS;
            let mut widgets = Vec::with_capacity(total_n_reg * 2);

            let empty_listener = fw::Listener::empty();

            let register_title_widget_generator = |s: &str| {
                let mut tb =
                    fw::text::TextBox::new(s, empty_listener.clone(), fw::text::TextAlign::Left);
                tb.set_style(
                    cgi::text_formatting::attributes::BOLD | cgi::text_formatting::colors::GREY,
                );
                Widget::new(tb)
            };
            let register_value_widget_generator = || {
                let rand = 0xFFFF;
                let tb = fw::text::TextBox::new(
                    &format!("0x{:04X}", rand),
                    empty_listener.clone(),
                    fw::text::TextAlign::Left,
                );

                Widget::new(tb)
            };

            for i in 0..total_n_reg {
                let rname = match i {
                    0..3 => &format!("R{}: ", i),
                    3..6 => &format!("A{}: ", i - 3),
                    6 => "z",
                    7 => "f",
                    8 => "pc",
                    9 => "sp",
                    _ => unreachable!(),
                };
                let register_title_widget = register_title_widget_generator(rname);
                widgets.push(register_title_widget);
            }

            for _ in 0..total_n_reg {
                let register_title_widget = register_value_widget_generator();
                widgets.push(register_title_widget);
            }

            let outline = WidgetBuilder::new(fw::utils::Empty)
                .with_outline(cgi::symbols::OutlineStyle::Normal)
                .with_title("Registers")
                .build();

            Self {
                widgets,
                outline,
                registers: [0; _],
            }
        }

        pub fn add_to_layout(&self, placement: WidgetPlacement, layout: &mut Layout) {
            let mut registers_split = [WidgetPlacement::default(); 12];
            placement
                .expand_or_shrink(-1, -1)
                .shift(1, 0)
                .split(3, 4, true, &mut registers_split);
            let register_titles_placements = registers_split
                .iter()
                .map(|p| p.with_width(4))
                .take(Self::NUM_REGISTERS + Self::NUM_SPECIAL_REGISTERS);
            let register_values_placements = registers_split
                .iter()
                .map(|p| p.shift_top_left(4, 0))
                .take(Self::NUM_REGISTERS + Self::NUM_SPECIAL_REGISTERS);

            let mut iter = self.widgets.iter();
            for p in register_titles_placements {
                layout.add_widget(&iter.next().unwrap(), p);
            }
            for p in register_values_placements {
                layout.add_widget(&iter.next().unwrap(), p);
            }

            layout.add_widget(&self.outline, placement);
        }
    }

    impl MessageBoards {
        pub fn new() -> Self {
            use cgi::text_formatting::*;

            let mut old_messages = msg_board::new("Old Messages");
            let mut new_messages = msg_board::new("New Messages");

            old_messages
                .edit()
                .set_style(attributes::DIM | attributes::ITALIC);
            new_messages.edit().set_style(attributes::ITALIC);

            for i in 0..3 {
                msg_board::add_message(&mut new_messages, &format!("NEW {}", i));
            }
            for i in 0..14 {
                msg_board::add_message(&mut old_messages, &format!("OLD {}", i));
            }

            Self {
                old_messages,
                new_messages,
            }
        }

        pub fn add_to_layout(
            &self,
            old_messages_placement: WidgetPlacement,
            new_messages_placement: WidgetPlacement,
            layout: &mut Layout,
        ) {
            layout.connect_and_add_widgets(
                vec![&self.old_messages, &self.new_messages],
                &mut [old_messages_placement, new_messages_placement],
            );
        }
    }
}

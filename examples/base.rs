use pushrod::base_widget::BaseWidget;
use pushrod::box_widget::BoxWidget;
use pushrod::engine::Engine;
use pushrod::geometry::{point, size};
use pushrod::widget::{SystemWidget, Widget};
use sdl2::pixels::Color;
use pushrod::text_widget::{TextJustify, TextWidget};

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("pushrod", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut engine = Engine::new(size(800, 600), 30);

    let mut base_widget = BaseWidget::new(point(20, 20), size(760, 560));
    base_widget.set_color(Color::RGBA(127, 127, 127, 255));
    let base_widget_id = engine.add_widget(SystemWidget::Base(Box::new(base_widget)));

    eprintln!("Added base widget ID: {}", base_widget_id);

    let mut text_widget = TextWidget::new(point(24, 24), size(300, 40),
        String::from("assets/OpenSans-Regular.ttf"), sdl2::ttf::FontStyle::NORMAL, 28,
        Color::BLACK, TextJustify::Left, String::from("Welcome to Pushrod"),
    );
    let text_widget_id1 = engine.add_widget(SystemWidget::Text(Box::new(text_widget)));

    eprintln!("Added text widget ID: {}", text_widget_id1);

    let mut box_widget1 = BoxWidget::new(point(40, 80), size(100, 100), Color::BLUE, 3);
    box_widget1.set_color(Color::CYAN);
    let box_widget_id1 = engine.add_widget(SystemWidget::Box(Box::new(box_widget1)));

    eprintln!("Added box widget ID: {}", box_widget_id1);

    let mut box_widget2 = BoxWidget::new(point(180, 80), size(100, 100), Color::GREEN, 5);
    box_widget2.set_color(Color::GRAY);
    let box_widget_id2 = engine.add_widget(SystemWidget::Box(Box::new(box_widget2)));

    eprintln!("Added box widget ID: {}", box_widget_id2);

    let mut box_widget3 = BoxWidget::new(point(320, 80), size(100, 100), Color::RED, 10);
    box_widget3.set_color(Color::MAGENTA);
    let box_widget_id3 = engine.add_widget(SystemWidget::Box(Box::new(box_widget3)));

    eprintln!("Added box widget ID: {}", box_widget_id3);

    // let mut button_widget1 = ButtonWidget::new(point(40, 160), size(140, 60));
    // let button_widget_id1 = engine.add_widget(SystemWidget::Button(Box::new(button_widget1)));
    //
    // eprintln!("Added button widget ID: {}", button_widget_id1);

    // let mut new_base_widget = BaseWidget::new(make_points(100, 100), make_size(600, 400));
    //
    // new_base_widget
    //     .get_config()
    //     .set_color(CONFIG_COLOR_BORDER, Color::RGB(0, 0, 0));
    // new_base_widget
    //     .get_config()
    //     .set_numeric(CONFIG_BORDER_WIDTH, 2);
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_entered(|x, _widgets, _layouts| {
    //         x.get_config()
    //             .set_color(CONFIG_COLOR_BASE, Color::RGB(255, 0, 0));
    //         x.get_config().set_invalidated(true);
    //         _widgets[0]
    //             .widget
    //             .borrow_mut()
    //             .get_config()
    //             .set_invalidated(true);
    //         eprintln!("Mouse Entered");
    //     });
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_exited(|x, _widgets, _layouts| {
    //         x.get_config()
    //             .set_color(CONFIG_COLOR_BASE, Color::RGB(255, 255, 255));
    //         x.get_config().set_invalidated(true);
    //         _widgets[0]
    //             .widget
    //             .borrow_mut()
    //             .get_config()
    //             .set_invalidated(true);
    //         eprintln!("Mouse Exited");
    //     });
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_moved(|_widget, _widgets, _layouts, points| {
    //         eprintln!("Mouse Moved: {:?}", points);
    //     });
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_scrolled(|_widget, _widgets, _layouts, points| {
    //         eprintln!("Mouse Scrolled: {:?}", points);
    //     });
    //
    // new_base_widget.get_callbacks().on_mouse_clicked(
    //     |_widget, _widgets, _layouts, button, clicks, state| {
    //         eprintln!(
    //             "Mouse Clicked: button={} clicks={} state={}",
    //             button, clicks, state
    //         );
    //     },
    // );
    //
    // engine.add_widget(Box::new(new_base_widget), String::from("widget1"));
    //
    // engine.on_exit(|engine| {
    //     let buttons: Vec<_> = vec![
    //         ButtonData {
    //             flags: MessageBoxButtonFlag::RETURNKEY_DEFAULT,
    //             button_id: 1,
    //             text: "Yes",
    //         },
    //         ButtonData {
    //             flags: MessageBoxButtonFlag::ESCAPEKEY_DEFAULT,
    //             button_id: 2,
    //             text: "No",
    //         },
    //     ];
    //
    //     let res = show_message_box(
    //         MessageBoxFlag::WARNING,
    //         buttons.as_slice(),
    //         "Quit",
    //         "Are you sure?",
    //         None,
    //         None,
    //     )
    //         .unwrap();
    //
    //     if let ClickedButton::CustomButton(x) = res {
    //         if x.button_id == 1 {
    //             return true;
    //         }
    //     }
    //
    //     false
    // });

    engine.run(sdl_context, window);
}

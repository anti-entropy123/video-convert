use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use iced::alignment;
use iced::executor;
use iced::subscription;
use iced::widget::{button, container, text, Column};
use iced::window;
use iced::window::Event as WindowEvent;
use iced::Event;
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription, Theme};

const FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resource/SourceHanSansCN-Regular.otf"
));

pub fn main() -> iced::Result {
    VideoProcessor::run(Settings {
        default_font: Some(FONT),
        window: window::Settings {
            size: (600, 400),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, Default, Clone)]
struct SelectTargetCtx {
    video: PathBuf,
}

#[derive(Debug, Default, Clone)]
struct CompleteCtx {
    target_path: PathBuf,
}

#[derive(Debug, Default)]
enum VideoProcessor {
    #[default]
    SelectFile,
    SelectTarget(SelectTargetCtx),
    GeneratingFile,
    Complete(CompleteCtx),
    Error,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    Submit(String),
    FfmpegComplete(PathBuf),
    FfmpegFound(bool),
}

impl Application for VideoProcessor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (VideoProcessor, Command<Message>) {
        (
            VideoProcessor::default(),
            Command::perform(ffmpeg_found(), |is_exist| Message::FfmpegFound(is_exist)),
        )
    }

    fn title(&self) -> String {
        String::from("视频格式转换器 - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if let VideoProcessor::Error = self {
            return Command::none();
        };

        match message {
            Message::EventOccurred(event) => {
                let file_path = if let Event::Window(WindowEvent::FileDropped(file)) = event {
                    println!("{}", file.to_str().unwrap());
                    file
                } else {
                    return Command::none();
                };
                if !file_path.is_file() {
                    return Command::none();
                }
                *self = VideoProcessor::SelectTarget(SelectTargetCtx { video: file_path });
                Command::none()
            }
            Message::Submit(video_type) => {
                let cur_ctx = if let VideoProcessor::SelectTarget(ctx) = self {
                    ctx.clone()
                } else {
                    panic!("Wrong application state.")
                };

                *self = VideoProcessor::GeneratingFile;
                Command::perform(ffmpeg_execute(cur_ctx.video, video_type), |path: PathBuf| {
                    Message::FfmpegComplete(path)
                })
            }
            Message::FfmpegComplete(p) => {
                *self = VideoProcessor::Complete(CompleteCtx { target_path: p });

                Command::none()
            }
            Message::FfmpegFound(is_exist) if !is_exist => {
                *self = VideoProcessor::Error;

                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        match self {
            VideoProcessor::SelectFile => select_file_view(),
            VideoProcessor::SelectTarget(ctx) => select_target_view(ctx),
            VideoProcessor::GeneratingFile => gen_file_view(),
            VideoProcessor::Complete(ctx) => complete_view(&ctx.target_path),
            VideoProcessor::Error => error_view(),
        }
    }
}

fn select_file_view() -> Element<'static, Message> {
    let txt = text("将源文件拖拽至此")
        .width(100)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center);

    let content = Column::new()
        .align_items(Alignment::Center)
        .spacing(20)
        .push(txt);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn select_target_view(ctx: &SelectTargetCtx) -> Element<'static, Message> {
    let txt = text(&format!(
        "要将 {} 转为:",
        ctx.video
            .file_name()
            .expect("bad file name?")
            .to_string_lossy()
    ))
    .size(28);

    let button_mp4 = button(
        text("MP4")
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center),
    )
    .width(Length::Fixed(100.))
    .on_press(Message::Submit("mp4".to_string()));

    let button_gif = button(
        text("GIF")
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center),
    )
    .width(Length::Fixed(100.))
    .on_press(Message::Submit("gif".to_string()));

    let content = Column::new()
        .align_items(Alignment::Center)
        .spacing(5)
        .push(txt)
        .push(button_mp4)
        .push(button_gif);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn gen_file_view() -> Element<'static, Message> {
    let txt = text("转换中...")
        .width(100)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center);

    let content = Column::new()
        .align_items(Alignment::Center)
        .spacing(20)
        .push(txt);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn complete_view(dst_path: &Path) -> Element<'static, Message> {
    let txt = text("转换完成")
        .width(100)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center);

    let path_txt = text(dst_path.to_str().unwrap())
        .width(100)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center);

    let content = Column::new()
        .align_items(Alignment::Center)
        .spacing(20)
        .push(txt)
        .push(path_txt);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn error_view() -> Element<'static, Message> {
    let txt = text("未安装ffmpeg!")
        .size(28)
        .width(100)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center);

    let content = Column::new()
        .align_items(Alignment::Center)
        .spacing(20)
        .push(txt);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn _ffmpeg_execute(src_video: PathBuf, video_type: String) -> PathBuf {
    let src_path = src_video.to_str().unwrap();
    let dst_path = {
        let filename_without_suffix = src_video
            .file_stem()
            .map(|name| name.to_str().expect("bad file path"))
            .unwrap_or("output");

        let dir = PathBuf::new()
            .join("./")
            .join("test/dist");

        println!("dir: {}", dir.to_str().unwrap());
        if dir.is_file() {
            panic!("already has file.")
        }
        if !dir.is_dir() {
            fs::create_dir(dir.clone()).expect("create dst dir failed.");
        }
        if video_type == "mp4" {
            dir.join(filename_without_suffix.to_owned() + ".mp4")
        } else {
            dir.join(filename_without_suffix.to_owned() + ".gif")
        }
    };

    if dst_path.is_file() {
        fs::remove_file(dst_path.clone()).expect("remove file failed.")
    }

    let mut command = process::Command::new("ffmpeg");
    if video_type == "mp4" {
        command
        .arg("-i")
        .arg(src_path)
        .arg("-vf")
        .arg("scale=trunc(iw/2)*2:trunc(ih/2)*2")
        .arg(dst_path.clone());
    } else {
        command
        .arg("-i")
        .arg(src_path)
        .arg(dst_path.clone());
    }

    println!("{:?}", command);
    let result = command.output().expect("ffmpeg execute failed.");
    let output = if result.status.success() {
        result.stdout
    } else {
        result.stderr
    };

    println!("{}", String::from_utf8(output).unwrap());
    dst_path
}

#[test]
fn test_ffmpeg_execute() {
    _ffmpeg_execute(
        PathBuf::new().join("/mnt/yjn/DATA/Videos/录屏/录屏 2023年04月17日 19时13分35秒.webm"),
        "mp4".to_string()
    );
}

async fn ffmpeg_execute(src_video: PathBuf, video_type: String) -> PathBuf {
    if video_type == "mp4" {
        _ffmpeg_execute(src_video, video_type)
    } else {
        _ffmpeg_execute(src_video, video_type)
    }
    
}

fn _ffmpeg_found() -> bool {
    let output = process::Command::new("ffmpeg").arg("-version").output();

    output.is_ok() && output.unwrap().status.success()
}

#[test]
fn test_ffmpeg_found() {
    assert!(_ffmpeg_found())
}

async fn ffmpeg_found() -> bool {
    _ffmpeg_found()
}

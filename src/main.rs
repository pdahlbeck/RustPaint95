//! Prototype window for RustPaint95. Historical fidelity is NOT yet verified.
use font8x8::{UnicodeFonts, BASIC_FONTS};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use rustpaint95::{Canvas, Rgb};

const W: usize = 800;
const H: usize = 600;
const BG: u32 = 0xC0C0C0;
const DARK: u32 = 0x808080;
const WHITE: u32 = 0xFFFFFF;
const BLACK: u32 = 0;
const BLUE: u32 = 0x000080;
const LEFT: i32 = 61;
const TOP: i32 = 58;
const CW: i32 = 715;
const CH: i32 = 490;

const COLORS: [Rgb; 28] = [
    rgb(0,0,0),rgb(128,128,128),rgb(128,0,0),rgb(128,128,0),
    rgb(0,128,0),rgb(0,128,128),rgb(0,0,128),rgb(128,0,128),
    rgb(128,128,64),rgb(0,64,64),rgb(0,128,255),rgb(0,64,128),
    rgb(64,0,255),rgb(128,64,0),rgb(255,255,255),rgb(192,192,192),
    rgb(255,0,0),rgb(255,255,0),rgb(0,255,0),rgb(0,255,255),
    rgb(0,0,255),rgb(255,0,255),rgb(255,255,128),rgb(0,255,128),
    rgb(128,255,255),rgb(128,128,255),rgb(255,0,128),rgb(255,128,64),
];
const fn rgb(r: u8,g: u8,b: u8)->Rgb { Rgb{r,g,b} }
fn value(c: Rgb)->u32 { ((c.r as u32)<<16)|((c.g as u32)<<8)|c.b as u32 }

struct Screen { pixels: Vec<u32> }
impl Screen {
    fn new()->Self { Self{pixels:vec![BG;W*H]} }
    fn dot(&mut self,x:i32,y:i32,color:u32){
        if x>=0 && y>=0 && x<W as i32 && y<H as i32 {
            self.pixels[y as usize*W+x as usize]=color;
        }
    }
    fn rect(&mut self,x:i32,y:i32,w:i32,h:i32,c:u32){
        let x0=x.max(0);let y0=y.max(0);
        let x1=(x+w).min(W as i32);let y1=(y+h).min(H as i32);
        for yy in y0..y1 { for xx in x0..x1 { self.dot(xx,yy,c); } }
    }
    fn bevel(&mut self,x:i32,y:i32,w:i32,h:i32,sunken:bool){
        let (tl,br)=if sunken {(DARK,WHITE)}else{(WHITE,DARK)};
        self.rect(x,y,w,1,tl);self.rect(x,y,1,h,tl);
        self.rect(x,y+h-1,w,1,br);self.rect(x+w-1,y,1,h,br);
    }
    fn text(&mut self,x:i32,y:i32,s:&str,fg:u32){
        let mut xx=x;
        for ch in s.chars(){
            if let Some(glyph)=BASIC_FONTS.get(ch){
                for (row,bits) in glyph.iter().enumerate(){
                    for col in 0..8 {
                        if bits & (1<<col)!=0 {self.dot(xx+col,y+row as i32,fg);}
                    }
                }
            }
            xx+=8;
        }
    }
}
#[derive(Clone,Copy,PartialEq,Eq)]
enum Tool { Pencil, Line, Rectangle, Eraser }
impl Tool {
    fn label(self)-> &'static str {
        match self {Self::Pencil=>"Pencil",Self::Line=>"Line",Self::Rectangle=>"Rectangle",Self::Eraser=>"Eraser"}
    }
}
struct App {
    canvas: Canvas, tool: Tool, fg: Rgb, bg: Rgb,
    dragging: bool, start: (i32,i32), last: (i32,i32),
    status: String,
}
impl App {
    fn new()->Self {
        Self{canvas:Canvas::new(CW as u32,CH as u32).expect("valid canvas"),
            tool:Tool::Pencil,fg:Rgb::BLACK,bg:Rgb::WHITE,
            dragging:false,start:(0,0),last:(0,0),
            status:"Ready".to_owned()}
    }
    fn canvas_point(pos:(f32,f32))->Option<(i32,i32)> {
        let (x,y)=(pos.0 as i32-LEFT,pos.1 as i32-TOP);
        (x>=0 && x<CW && y>=0 && y<CH).then_some((x,y))
    }
    fn choose(&mut self,pos:(f32,f32),right:bool) {
        let (x,y)=(pos.0 as i32,pos.1 as i32);
        if x>=7 && x<55 && y>=59 && y<179 {
            // Only four tools are implemented so far. Other toolbar slots are deliberately inactive.
            let col=(x-7)/24;let row=(y-59)/24;
            self.tool=match (row,col){
                (0,0)=>Tool::Pencil,(0,1)=>Tool::Line,
                (1,0)=>Tool::Rectangle,(1,1)=>Tool::Eraser,
                _=>self.tool
            };
            self.status=format!("{} selected",self.tool.label());
        } else if y>=559 && y<591 && x>=79 && x<79+14*24 {
            let col=(x-79)/24; let row=(y-559)/16;
            let i=(row*14+col) as usize;
            if let Some(color)=COLORS.get(i).copied() {
                if right{self.bg=color;}else{self.fg=color;}
            }
        }
    }
    fn gesture_start(&mut self,p:(i32,i32),right:bool){
        self.canvas.begin_action();self.start=p;self.last=p;self.dragging=true;
        let c=if right{self.bg}else{self.fg};
        match self.tool {
            Tool::Pencil=>self.canvas.put_pixel(p.0,p.1,c),
            Tool::Eraser=>self.canvas.put_pixel(p.0,p.1,self.bg),
            _=>{}
        }
    }
    fn gesture_move(&mut self,p:(i32,i32),right:bool){
        let color=if self.tool==Tool::Eraser {self.bg}
            else if right {self.bg}else{self.fg};
        match self.tool {
            Tool::Pencil|Tool::Eraser=>self.canvas.line(self.last.0,self.last.1,p.0,p.1,color),
            _=>{}
        }
        self.last=p;
    }
    fn gesture_end(&mut self,right:bool){
        let color=if right{self.bg}else{self.fg};
        match self.tool {
            Tool::Line=>self.canvas.line(self.start.0,self.start.1,self.last.0,self.last.1,color),
            Tool::Rectangle=>{
                let (x0,y0)=self.start;let (x1,y1)=self.last;
                self.canvas.line(x0,y0,x1,y0,color);
                self.canvas.line(x1,y0,x1,y1,color);
                self.canvas.line(x1,y1,x0,y1,color);
                self.canvas.line(x0,y1,x0,y0,color);
            }
            _=>{}
        }
        self.dragging=false;
    }
    fn render(&self,s:&mut Screen){
        s.rect(0,0,W as i32,H as i32,BG);
        s.rect(0,0,W as i32,23,BLUE);
        s.text(9,7,"RustPaint95 - untitled",WHITE);
        s.text(9,32,"File   Edit   View   Image   Options   Help",BLACK);
        s.rect(57,54,723,498,DARK);
        s.bevel(57,54,723,498,true);
        s.rect(LEFT,TOP,CW,CH,WHITE);
        for y in 0..CH {for x in 0..CW{
            if let Some(c)=self.canvas.pixel(x,y){s.dot(LEFT+x,TOP+y,value(c));}
        }}
        for row in 0..8 {
            for col in 0..2 {
                let x=7+col*24;let y=59+row*24;
                s.rect(x,y,23,23,BG);
                s.bevel(x,y,23,23,false);
                let label=match (row,col){
                    (0,0)=>"P",(0,1)=>"/",(1,0)=>"R",(1,1)=>"E",
                    (2,0)=>"?",(2,1)=>"?",(3,0)=>"?",(3,1)=>"?",
                    _=>"."
                };
                s.text(x+8,y+7,label,if row<2 {BLACK}else{DARK});
            }
        }
        let tool_idx=match self.tool{Tool::Pencil=>0,Tool::Line=>1,Tool::Rectangle=>2,Tool::Eraser=>3};
        let bx=7+(tool_idx%2)*24;let by=59+(tool_idx/2)*24;
        s.bevel(bx,by,23,23,true);
        s.rect(4,558,70,34,BG);s.bevel(4,558,70,34,true);
        s.rect(27,562,33,25,value(self.bg));s.rect(13,566,33,22,value(self.fg));
        s.bevel(13,566,33,22,true);
        for (i,c) in COLORS.iter().enumerate(){
            let x=79+(i%14) as i32*24;let y=559+(i/14) as i32*16;
            s.rect(x,y,22,15,DARK);s.rect(x+2,y+2,18,11,value(*c));
        }
        s.rect(0,548,W as i32,2,WHITE);
        s.rect(0,592,W as i32,1,WHITE);
        s.text(423,568,"RustPaint95 prototype",BLACK);
        s.text(8,594,&self.status.chars().take(70).collect::<String>(),BLACK);
    }
}
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let mut window=Window::new("RustPaint95 - prototype",W,H,WindowOptions{
        resize:false,scale:Scale::X1,..WindowOptions::default()
    })?;
    window.set_target_fps(60);
    let mut s=Screen::new();let mut app=App::new();
    let mut prev_down=false;
    let mut gesture_right=false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let control=window.is_key_down(Key::LeftCtrl)||window.is_key_down(Key::RightCtrl)
            ||window.is_key_down(Key::LeftSuper)||window.is_key_down(Key::RightSuper);
        if control && window.is_key_pressed(Key::Z,KeyRepeat::No){
            if app.canvas.undo(){app.status="Undo".into();}
        }
        if control && window.is_key_pressed(Key::Y,KeyRepeat::No){
            if app.canvas.redo(){app.status="Redo".into();}
        }
        if control && window.is_key_pressed(Key::S,KeyRepeat::No){
            match app.canvas.to_bmp24().and_then(|data|
                std::fs::write("rustpaint95.bmp",data).map_err(|_|"could not save")
            ){
                Ok(())=>app.status="Saved rustpaint95.bmp".into(),
                Err(e)=>app.status=e.into()
            }
        }
        let left=window.get_mouse_down(MouseButton::Left);
        let right=window.get_mouse_down(MouseButton::Right);
        let down=left||right;
        if let Some(pos)=window.get_mouse_pos(MouseMode::Clamp){
            if down && !prev_down {
                gesture_right=right;
                if let Some(p)=App::canvas_point(pos){app.gesture_start(p,gesture_right);}
                else {app.choose(pos,gesture_right);}
            } else if down && app.dragging {
                if let Some(p)=App::canvas_point(pos){app.gesture_move(p,gesture_right);}
            }
        }
        if !down && prev_down && app.dragging { app.gesture_end(gesture_right); }
        prev_down=down;
        app.render(&mut s);
        window.update_with_buffer(&s.pixels,W,H)?;
    }
    Ok(())
}

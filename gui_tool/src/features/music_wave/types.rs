//! 类型定义：音阶、和弦、消息、状态

use iced::Color;
use std::collections::HashSet;
use std::f64::consts::TAU;
use std::time::Instant;

pub const A4_FREQ: f64 = 440.0;
pub const AMPLITUDE: f64 = 0.8;
pub const PERIOD_SECONDS: f64 = 3.0;
pub const DEFAULT_SPEED: f32 = 1.0;

// ============================================================
// 音名
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PitchClass { C, Cs, D, Ds, E, F, Fs, G, Gs, A, As, B }

impl PitchClass {
    pub const ALL: [PitchClass; 12] = [
        PitchClass::C,PitchClass::Cs,PitchClass::D,PitchClass::Ds,
        PitchClass::E,PitchClass::F,PitchClass::Fs,PitchClass::G,
        PitchClass::Gs,PitchClass::A,PitchClass::As,PitchClass::B,
    ];
    pub fn semitone(self) -> i32 {
        match self {
            PitchClass::C=>-9,PitchClass::Cs=>-8,PitchClass::D=>-7,PitchClass::Ds=>-6,
            PitchClass::E=>-5,PitchClass::F=>-4,PitchClass::Fs=>-3,PitchClass::G=>-2,
            PitchClass::Gs=>-1,PitchClass::A=>0,PitchClass::As=>1,PitchClass::B=>2,
        }
    }
    pub fn label(self)->&'static str {
        match self {
            PitchClass::C=>"C",PitchClass::Cs=>"C#",PitchClass::D=>"D",PitchClass::Ds=>"D#",
            PitchClass::E=>"E",PitchClass::F=>"F",PitchClass::Fs=>"F#",PitchClass::G=>"G",
            PitchClass::Gs=>"G#",PitchClass::A=>"A",PitchClass::As=>"A#",PitchClass::B=>"B",
        }
    }
    pub fn color(self)->Color {
        let h=(self.semitone().rem_euclid(12)as f32)*30.0*std::f32::consts::PI/180.0;
        Color::from_rgb(0.5+0.5*(h+2.094).cos(),0.5+0.5*(h-2.094).cos(),0.5+0.5*h.cos())
    }
}

// ============================================================
// 八度
// ============================================================

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum Octave { Three, Four, Five }

impl Octave {
    pub const ALL:[Octave;3]=[Octave::Three,Octave::Four,Octave::Five];
    pub fn number(self)->i32 { match self { Octave::Three=>3,Octave::Four=>4,Octave::Five=>5 } }
    pub fn label(self)->&'static str { match self { Octave::Three=>"3",Octave::Four=>"4",Octave::Five=>"5" } }
}

// ============================================================
// 完整音高
// ============================================================

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Pitch { pub class: PitchClass, pub octave: Octave }

impl Pitch {
    pub fn all()->Vec<Pitch> {
        let mut v=Vec::with_capacity(36);
        for &o in &Octave::ALL { for &c in &PitchClass::ALL { v.push(Pitch{class:c,octave:o}); } }
        v
    }
    pub fn semitone_offset(self)->i32 { (self.octave.number()-4)*12+self.class.semitone() }
    pub fn frequency(self)->f64 { super::types::A4_FREQ*(2.0f64).powf(self.semitone_offset() as f64/12.0) }
    pub fn label(self)->String { format!("{}{}",self.class.label(),self.octave.label()) }
    pub fn color(self)->Color { self.class.color() }
}

// ============================================================
// 和弦
// ============================================================

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ChordType {
    Major,Minor,Augmented,Diminished,
    Dominant7,Major7,Minor7,
    Major9,Minor9,Sus4,Add9,
}

impl ChordType {
    pub fn intervals(self)->&'static[i32] { match self {
        ChordType::Major=>&[0,4,7],ChordType::Minor=>&[0,3,7],
        ChordType::Augmented=>&[0,4,8],ChordType::Diminished=>&[0,3,6],
        ChordType::Dominant7=>&[0,4,7,10],ChordType::Major7=>&[0,4,7,11],
        ChordType::Minor7=>&[0,3,7,10],ChordType::Major9=>&[0,4,7,11,14],
        ChordType::Minor9=>&[0,3,7,10,14],ChordType::Sus4=>&[0,5,7],
        ChordType::Add9=>&[0,4,7,14],
    }}
    pub fn label(self)->&'static str { match self {
        ChordType::Major=>"大三",ChordType::Minor=>"小三",
        ChordType::Augmented=>"增三",ChordType::Diminished=>"减三",
        ChordType::Dominant7=>"属七",ChordType::Major7=>"大七",
        ChordType::Minor7=>"小七",ChordType::Major9=>"大九",
        ChordType::Minor9=>"小九",ChordType::Sus4=>"挂四",ChordType::Add9=>"加九",
    }}
    pub fn notes(self,root:Pitch)->Vec<Pitch> {
        let all=Pitch::all(); let ro=root.semitone_offset();
        self.intervals().iter().filter_map(|&iv| all.iter().find(|p| p.semitone_offset()==ro+iv).copied()).collect()
    }
}

// ============================================================
// 谐波基频
// ============================================================

fn semitone_ratio(semitones: i32) -> (i32, i32) {
    match semitones.rem_euclid(12) {
        0=>(1,1),1=>(16,15),2=>(9,8),3=>(6,5),4=>(5,4),5=>(4,3),
        6=>(7,5),7=>(3,2),8=>(8,5),9=>(5,3),10=>(9,5),11=>(15,8),
        _=>(1,1),
    }
}
fn gcd(mut a:i32,mut b:i32)->i32{while b!=0{(a,b)=(b,a%b);}a}
fn lcm(a:i32,b:i32)->i32{a/gcd(a,b)*b}

pub fn harmonic_base(pitches:&[Pitch])->f64{
    let min_off=pitches.iter().map(|p|p.semitone_offset()).min().unwrap_or(0);
    let f_min=pitches.iter().map(|p|p.frequency()).min_by(|a,b|a.partial_cmp(b).unwrap()).unwrap_or(A4_FREQ);
    let mut all_q=1i32;
    for p in pitches{let(_,q)=semitone_ratio(p.semitone_offset()-min_off);all_q=lcm(all_q,q);}
    f_min/all_q as f64
}

// ============================================================
// 消息
// ============================================================

#[derive(Debug,Clone)]
pub enum Msg {
    Tick(Instant),
    ToggleCheck(Pitch),
    SelectSolo(Pitch),
    Clear,
    SelectChord(ChordType),
    SoloSpeedChanged(f32),
    SoloZoom(f32),
    ComboSpeedChanged(f32),
    ComboZoom(f32),
}

// ============================================================
// 状态
// ============================================================

pub struct MusicWave {
    pub time: f64, pub last_tick: Option<Instant>,
    pub checked: HashSet<Pitch>, pub solo: Option<Pitch>,
    pub solo_speed: f32, pub solo_zoom: f32,
    pub combo_speed: f32, pub combo_zoom: f32,
}

impl Default for MusicWave {
    fn default()->Self{
        let mut c=HashSet::new();
        c.insert(Pitch{class:PitchClass::A,octave:Octave::Four});
        Self{
            checked:c,
            solo:Some(Pitch{class:PitchClass::A,octave:Octave::Four}),
            solo_speed:DEFAULT_SPEED, solo_zoom:1.0,
            combo_speed:DEFAULT_SPEED, combo_zoom:1.0,
            time:0.0, last_tick:None,
        }
    }
}

impl MusicWave {
    pub fn solo_phase(&self)->f64{self.time*TAU/PERIOD_SECONDS*self.solo_speed as f64}
    pub fn combo_phase(&self)->f64{self.time*TAU/PERIOD_SECONDS*self.combo_speed as f64}
    pub fn checked_freqs(&self)->Vec<f64>{let mut v:Vec<f64>=self.checked.iter().map(|p|p.frequency()).collect();v.sort_by(|a,b|a.partial_cmp(b).unwrap());v}
    pub fn checked_pitches(&self)->Vec<Pitch>{let mut v:Vec<Pitch>=self.checked.iter().copied().collect();v.sort();v}
}

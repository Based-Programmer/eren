# eren

Stream & Download Cartoons & Animes

# Install

#### Linux/Android

- Get the binary from the [release page](https://github.com/Based-Programmer/eren/releases)

## Build
#### Linux/Unix/Mac/Android

- First of all install rust then

````sh
git clone 'https://github.com/Based-Programmer/eren' && \
cd eren && \
cargo build --release
````

- Then move it to your $PATH

````sh
sudo cp target/release/eren /usr/local/bin/
````

- Or Build it directly with cargo

````sh
cargo install eren
````

- Then move it to your $PATH

````sh
sudo cp "$CARGO_HOME"/bin/eren /usr/local/bin/
````

- Or better add $CARGO_HOME to your $PATH

- In your .zprofile, .bash_profile or .fish_profile ?

````sh
export PATH="$CARGO_HOME/bin:$PATH"
````

#### Only Android Termux

- In your .zprofile, .bash_profile or .fish_profile ?

````sh
export TERMINFO='/data/data/com.termux/files/usr/share/terminfo'
````

## Usage

````
eren <args> <search query>
````

#### Example

- Get data

````sh
eren --debug demon slayer
 ````

- Change Provider

````sh
eren -p Luf-mp4 death note
````

####  Multi-select

- use Tab in the tui [skim](https://github.com/lotabout/skim)
- use Shift + Enter in [rofi](https://github.com/davatorium/rofi) 
- select 1 & 12 & it will play 1 to 12
- In mpv-android, play a episode first then multi-select using `select` option

#### Rofi

 - you can execute eren from something like rofi or dmenu & rofi will spawn automatically
 - or you can just execute it from the terminal using the normie way given below
    
 ````sh
eren -r texhnolyze
````

- Sort by top (best cope for occasional allanime's irrelevant search results) 

````sh
eren -t monster
````
  
- Sub
  
````sh
eren -s great pretender
````

- More at help

````sh
eren -h
````

## Dependency

- mpv or mpv-android (best media player)
- ffmpeg (merging downloaded video & audio)
- [hls](https://github.com/CoolnsX/hls_downloader) (for downloading m3u8 from provider Ak & Luf-mp4)

## Optimal Dependency

- rofi (external selection menu)

## Acknowledgement

- Heavily inspired from [ani-cli](https://github.com/pystardust/ani-cli)
- Special thanks to KR for decoding the [encryption](https://github.com/justfoolingaround/animdl/commit/c4e6a86)
- fuzzy tui [skim](https://github.com/lotabout/skim)

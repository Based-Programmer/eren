# eren

Stream & Download Cartoons & Animes

# Install

#### Linux/Mac

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

- Or Build it directly from crate

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
## Multi-Select

 - use Tab in the tui [skim](https://github.com/lotabout/skim)
 - use Shift + Enter in [rofi](https://github.com/davatorium/rofi)
 - just select the range like select 7 & 4 & it will play from 4 to 7

## Rofi

 - you can execute eren from something like rofi or dmenu & rofi will spawn automatically
 
 - or you can just execute it from the terminal using the normie way given below

       
 ````sh
eren -r texhnolyze
````

## Usage

````
eren <argument> <search query>
````

#### Example

- Get data

````sh
eren --debug demon slayer
 ````

- Change Provider

````sh
eren -p=S-mp4 death note
````

- Sort by top (best cope for occasion irrelevant search results of allanime)

````sh
eren -t monster
````
  
- Dub
  
````sh
eren -D great pretender
````

- More at help

````sh
eren -h
````

## Optimal Dependencies

- mpv (Streaming video)

## Acknowledgement
- Heavily inspired from [ani-cli](https://github.com/pystardust/ani-cli)
- Special thanks to KR for decoding the [encryption](https://github.com/justfoolingaround/animdl/commit/c4e6a86)
- fuzzy tui [skim](https://github.com/lotabout/skim)

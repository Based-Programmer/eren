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

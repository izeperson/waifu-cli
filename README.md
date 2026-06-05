# waifu-cli
prints things in a terminal, kinda cool.
built using rust and uses nekos.best api\
<img width="450" height="371" alt="waifu-cli" src="https://github.com/user-attachments/assets/129e5ff4-1623-41fe-973b-b89952a75291" />
# Installation (Arch Linux)
1. Install an AUR helper, [yay](https://github.com/Jguer/yay?tab=readme-ov-file#installation) for this example.
2. Run ```yay -S waifu```
3. Press enter on the prompts and wait for it to build.
4. After installation is completed, see [usage](https://github.com/izeperson/waifu-cli#Usage) on how to use the program.
# Installation (Other Distros + macOS)
1. Clone the git repository ```git clone https://github.com/izeperson/waifu-cli```
2. cd into "waifu-cli"
3. Build the project: ```cargo build --release``` (You may need to [install rust](https://rust-lang.org/tools/install/))
4. Create a symlink in `~/.local/bin`:
<pre>
  mkdir -p "$HOME/.local/bin"
  ln -sf "$PWD/target/release/waifu" "$HOME/.local/bin/waifu"
</pre>
5. Ensure `~/.local/bin` is in your `PATH`. After setup is completed, see [usage](https://github.com/izeperson/waifu-cli#Usage) on how to use the program.

(It may work in other Unix-like operating systems, like FreeBSD, though it is untested.)

# Usage
**Commands:**
<pre>
  Usage: waifu &lt;command&gt;
  Commands:
  -c, --category &lt;name&gt;   Fetch an image from a specific category
  -n, --batch &lt;amount&gt;    Use '-n &lt;amount&gt;' after category to batch download (e.g. -c waifu -n 50)
  -l, --list              List all available categories
  -r, --random            Fetch a random image from a random category
  -v, --version           Show version information
  -o                      Open the image URL in the default system viewer
  -t, --test              Test connectivity
  --min-size &lt;KB&gt;         Filter batch downloads by minimum file size
  --min-width &lt;pixels&gt;    Filter batch downloads by minimum width
  --min-height &lt;pixels&gt;   Filter batch downloads by minimum height
  --no-upscale            Don't upscale small images to fit the terminal
  --check-links           Perform a deep check of category endpoints
  -h, --help              Show the help message
</pre>
[kitty](https://sw.kovidgoyal.net/kitty/binary/) is the preferred terminal due to its ability to display images and animated GIFs.\
\
When running the program it will show a random (SFW) image based on the category you chose.\
\
Images are provided by the [nekos.best API](https://nekos.best/)\
\
You can use 's' to save the image (the image will saved in your working directory), 'u' to print the image URL, 'a' to see the artist, 'o' to open the image in your browser, 'n' to see another image and enter or 'q' to quit.\
<img width="544" height="692" alt="example of waifu -c waifu in the kitty terminal" src="https://github.com/user-attachments/assets/218ae03b-ea8b-4f1d-8cfa-aa2c2868043f" />

# waifu-cli
prints things in a terminal, kinda cool.
built using rust and uses waifu.pics api
<img width="450" height="371" alt="waifu-cli" src="https://github.com/user-attachments/assets/129e5ff4-1623-41fe-973b-b89952a75291" />
# Installation (Arch Linux)
1. Install an AUR helper, [yay](https://github.com/Jguer/yay?tab=readme-ov-file#installation) for this example.
2. Run ```yay -S waifu```
3. Press enter on the prompts and wait for it to build.
4. After installation is completed, use the ```waifu``` command to use the program.
# Installation (Other Distros)
1. Clone the git repository ```git clone https://github.com/izeperson/waifu-cli```
2. cd into "waifu-cli"
3. Run ```chmod +x run.sh``` to give executable perms to the sh file
4. Use ```sh run.sh``` or ```./run.sh``` to start setup.
5. After setup is completed, use the ```waifu``` command to use the program.

# Usage
**Commands:**
<pre>
  -c, --category (name)   Fetch an image from a specific category
  -l, --list              List all available categories
  -s, --stats             Show request performance statistics
  -t, --test              Test API connectivity
  -h, --help              Show the help message
</pre>
When running the program it will show a random (SFW) image based on the category you chose.\
You can use s to save the image (the image will be put in your current directory), n to see another image and enter or q to quit.\
<img width="968" height="832" alt="example of waifu -c smug" src="https://github.com/user-attachments/assets/5397fa64-e345-4504-9833-75d1af90eb38" />

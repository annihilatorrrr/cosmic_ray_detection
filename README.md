**Your computer can double up as a cosmic ray detector. Yes, really!**

[Cosmic rays](https://en.wikipedia.org/wiki/Cosmic_ray) hit your computer all the time. If they hit the RAM, this can [sometimes cause issues](https://en.wikipedia.org/wiki/Soft_error#Cosmic_rays_creating_energetic_neutrons_and_protons), like flipping a random bit in memory.
To use your computer as a cosmic ray detector, simply run this program!  
The detection works by allocating a vector of zeroed bytes and then checking regularly to see if they are all still zero. Ta-da!  

 * Do not run this on a computer with ECC memory, as that will prevent the issues we are trying to detect!
 * The more RAM you allocate the larger the probabillity of a bitflip occuring.
 * Beware of operating systems being clever, and e.g. compressing unused memory pages. A vector of nothing but zeros that hasn't been used in 30 seconds is an excellent target for this. This will shrink your detector!
 

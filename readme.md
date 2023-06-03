Multi User Container

Simple server accesible with clients (non web)

The goal of this server is to be able to securely store files on a remote machine with a multi account system

## Features 
- [ ] Accounts
- [ ] GUI
- [ ] File system on server's machine


This security protocol is obviously flawfull, any MiTM can read traffic.
Sending files through laminar::Packet and not stream is not optimal (i think)

rustls is too complex for now.
i don't want to make a webserver koz idk how make this work with a webserver (might be doable with rocket and sending files using get requests ?) idk this domain, my head hurts, fuck this i'll retry later

There might be something to do with the 'stream to socket' implementaton i did with the rustls lib

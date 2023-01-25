# jblomlof-sockets

## To run
Simply run server before you run client.
(I recommend building client (`cargo b`) and then running server (`cargo r`)

## Features
Not a lot. It's a live chatting forum with the following commands

##### Client side:
/ping -> Pong
/aboutme -> Retrieve your username and ID
/login -> login... /login \<username\> \<password\>
/create -> create account. 
/listall -> get the username of all active clients. 

###### Server side:
stop -> saves the data and stops the server.
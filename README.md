# jblomlof-sockets

## To run
Simply run server before you run client.
(I recommend building client (`cargo b`) and then running server (`cargo r`) and then ofcourse finding the executable client file)

## Features
Not a lot. It's a live chatting forum with the following commands

###### Client side:
/ping -> Pong  
/aboutme -> Retrieve your username and ID  
/login -> login..., prefer /login \<username\> \<password\>  
/create -> create account, prefer /create \<username\> \<password\>.  
/listall -> get the username of all active clients.   
/whisper -> send a direct message to a user. /whisper \<username\> \<message\>

###### Server side:
stop -> saves the data and stops the server.
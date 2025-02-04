![enter image description here](https://i.ibb.co/spgXq26h/Frame-2.png)
---
>**DockerBeam** is a P2P app that transfers Docker images directly between users.

# Usage
```dockerbeam [COMMAND] [ARGS] [OPTIONS]```

##### Commands :

```send/push <image>``` Push Docker image to another peer

```get/pull @<peer> ``` Pull Docker image from a peer

  

#### Options :

```--verbose``` Show detailed operation logs (level: Info)

```--verbose-max``` Show all debug logs (level : debug)

  

#### examples :
```rust
dockerbeam push rust:latest 		#Send a specific image

dockerbeam send 					#Select image interactively

dockerbeam pull @MTI3LjAuMC4x 		#Pull from peer MTI3LjAuMC4x

dockerbeam get 						#Pull from peer
```
# Installation

 - MacOS
  ```brew install ?```
  
  - Windows
  ```winget install ?```

- Linux 
	- AUR
	```pacman```
	- RPM
	 ```?```
	 - another one here
	 ```?```


---

> [!WARNING]
> Known Issues : 
> [https://github.com/DockerBeam/DockerBeam/issues/1]

  

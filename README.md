

![Frame 2](https://github.com/user-attachments/assets/e47afb1d-edab-4ab5-a705-13bd0d6716c7)
---
[![dockerbeam](https://snapcraft.io/dockerbeam/badge.svg)](https://snapcraft.io/dockerbeam)
>**DockerBeam** is a P2P app that transfers Docker images directly between users.



![gh](https://github.com/user-attachments/assets/5a34d156-06a7-487f-b674-54903a9de44f)  
[![Get it from the Snap Store](https://snapcraft.io/en/dark/install.svg)](https://snapcraft.io/dockerbeam)
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
dockerbeam push rust:latest 				#Send a specific image

dockerbeam send 					#Select image interactively

dockerbeam pull @MTI3LjAuMC4x 				#Pull from peer MTI3LjAuMC4x

dockerbeam get 						#Pull from peer
```
# Installation

 - MacOS
  ```brew tap dockerbeam/dockerbeam && brew install dockerbeam```
  
  - Windows
  ```winget install dockerbeam```

- Linux 
```curl --proto '=https' --tlsv1.2 -sSLf https://www.dockerbeam.com/linux | bash```
---

> [!WARNING]
> Known Issues : 
> [https://github.com/DockerBeam/DockerBeam/issues/1]

  ---
  For any issues , improvements , requests - Feel free to open up an [issue](https://github.com/DockerBeam/DockerBeam/issues)
  
Documentation regarding contributions , self host etc will be updated soon...




Thank you for reading till here.

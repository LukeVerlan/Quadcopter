## Documentation

Manual for the x4r 

[Documentation](https://www.lindinger.at/media/a3/67/07/1619033849/9721187_EN_Manual.pdf)

<img width="586" height="617" alt="image" src="https://github.com/user-attachments/assets/c33f3b71-437b-4cb7-87ea-0a0ad720635b" />

Hardware setup 

<img width="523" height="487" alt="image" src="https://github.com/user-attachments/assets/e7e84a55-b405-4ca5-9cf6-b39544ed5c27" />

The pins on the edge can be used to power the device as well as power peripherals. Setting up the device as shown below resulted in the following data coming from the SBUS pin 

<img width="696" height="207" alt="image" src="https://github.com/user-attachments/assets/7a27b21c-33c2-441c-bb2a-c27340208cae" />

Evidently, the signal is inverted by default, meaning a hardware inverter is needed to parse the signal as USART. 

## Hardware Inverter

<img width="1152" height="503" alt="image" src="https://github.com/user-attachments/assets/d93a37d9-93ef-4a21-a8ba-5c794f5919ca" />

After applying this hardware inverter to the output, this was the signal

<img width="662" height="278" alt="image" src="https://github.com/user-attachments/assets/3bf43990-f4e6-4376-ac04-a6f11d56e462" />

Then, after binding the transceiver, this was the final result

<img width="1638" height="511" alt="image" src="https://github.com/user-attachments/assets/881e5dcd-90b6-4920-90c6-38e007fad8e4" />

## Transceiver Channel Binds 

# Sent Value Range 
(172 = -100%, 992 = 0%, 1811 = 100%)

| Channel # | Output     |
| --------- | ---------- |
| 1         | Throttle   |
| 2         | Roll       | 
| 3         | Pitch      | 
| 4         | Yaw        | 






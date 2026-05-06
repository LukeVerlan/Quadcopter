## Quadcopter

_DRONE PROJECT 2026 -- Luke Verlangieri, Ashton Garcia_

**System Requirements**
- 

_Minimum Viable Requirements_
- Stable Flight 
  - Handle Disturbances / Noise
- Manual Control
- Basic Telemetry
- 2-3 lbs Payload

_Advanced Goals_
- Autonomous Flight
  - GPS Controlled
  - Obstacle avoidance 
- FPV
- Real Time Streaming

**Schedule -- Rough**
-

- 5/13 -> Ali express orders go out 
- 6/13 -> Have initial parts/hardware at house
- 7/13 -> Drivers written and tested
- 8/13 -> PCBs created and ordered
- 9/1 -> First tests
- 9/23 -> Integration of advanced features


**Hardware Components**
-
- Micro Controller (STM32F411CE) (https://www.st.com/en/microcontrollers-microprocessors/stm32f411ce.html) -- (https://www.aliexpress.us/item/3256810394123219.html?spm=a2g0o.productlist.main.2.5a3d226dsJZVKT&algo_pvid=7943a65c-7828-47f9-b61f-15ef49972387&algo_exp_id=7943a65c-7828-47f9-b61f-15ef49972387-1&pdp_ext_f=%7B%22order%22%3A%22226%22%2C%22eval%22%3A%221%22%2C%22fromPage%22%3A%22search%22%7D&pdp_npi=6%40dis%21USD%214.31%210.99%21%21%2129.26%216.70%21%402101d49617780375933226575e4027%2112000052882848401%21sea%21US%210%21ABX%211%210%21n_tag%3A-29910%3Bd%3A156c235e%3Bm03_new_user%3A-29895%3BpisId%3A5000000203531301&curPageLogUid=YchjBW2zcJxP&utparam-url=scene%3Asearch%7Cquery_from%3A%7Cx_object_id%3A1005010580437971%7C_p_origin_prod%3A) 
- IMU (ICM-42688-P) (https://d17t6iyxenbwp1.cloudfront.net/s3fs-public/2026-02/ds-000347_icm-42688-p-datasheet_0.pdf?VersionId=z2Bv_vW3nu7NZg3E3TYHbENt_fuKQupW) -- (https://www.aliexpress.us/item/3256807957020127.html?src=google&gatewayAdapt=glo2usa#nav-review)
- GPS (NEO-M8N) (https://content.u-blox.com/sites/default/files/NEO-M8-FW3_DataSheet_UBX-15031086.pdf) -- (https://www.aliexpress.us/item/3256808489757098.html?spm=a2g0o.productlist.main.1.346b225eTRbKp6&algo_pvid=f3da2296-f1be-4608-9f8a-14be93721ba2&algo_exp_id=f3da2296-f1be-4608-9f8a-14be93721ba2-0&pdp_ext_f=%7B%22order%22%3A%22161%22%2C%22eval%22%3A%221%22%2C%22fromPage%22%3A%22search%22%7D&pdp_npi=6%40dis%21USD%2141.80%2110.90%21%21%21283.56%2173.94%21%402101e80317780365815084457e64bb%2112000046498390213%21sea%21US%210%21ABX%211%210%21n_tag%3A-29910%3Bd%3A156c235e%3Bm03_new_user%3A-29895%3BpisId%3A5000000206001318&curPageLogUid=w60hsF6ggFS1&utparam-url=scene%3Asearch%7Cquery_from%3A%7Cx_object_id%3A1005008676071850%7C_p_origin_prod%3A)
- Altimeter (BMP388) (https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmp388-ds001.pdf) -- (https://www.adafruit.com/product/3966?srsltid=AfmBOor6PiHuxDTMsAyVgUWm70ycKX0sTWk2rwKNLSaXOjRAn6OCI0IP)
- Flashchip (w25q128jv) (https://www.mouser.com/datasheet/2/949/w25q128jv_revf_03272018_plus-1489608.pdf?srsltid=AfmBOoozvsA8aRRo3I--P_RtC70a9COQJ-V8uNmXnQTpi-3oZV0jr_qi) -- (https://www.adafruit.com/product/5643?srsltid=AfmBOoqDm2NeXzhQkg1-jKzEhoKhAl4ELXjaFW-NJGIZweRUNaKCwcr6)

import os
from PIL import Image

base = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills"

def check_frame(f, x1, y1, x2, y2, desc):
    img = Image.open(os.path.join(base, f"g{f:04d}.png")).convert("RGB")
    print(f"g{f} {desc}:", img.getpixel((x1, y1)), img.getpixel((x2, y2)))

# g0520 - chart cold start (local 100)
check_frame(520, 420, 378, 1580, 54, "chart cold start - hero bar at x420")

# g0900 - Robo vignette (local 60)
check_frame(900, 420, 556, 1580, 62, "Robo vignette - hero bar at x420")

# g0960 - Fermat hold (local 870)
check_frame(960, 260, 536, 360, 100, "Fermat hold - dot cluster at x260,y536")

# g1000 - Fermat payoff (local 1130)
check_frame(1000, 1130, 500, 520, 60, "Fermat payoff - clock at x1130,y500")

# g1300 - hero -20% (local 1264)
check_frame(1300, 1310, 630, 1520, 100, "Economics hero -20% at x1310,y630")

# g1420 - hero settling (local 1420)
check_frame(1420, 1330, 604, 1520, 50, "Economics hero settling at x1330,y604")

# g1700 - lockup (local 1700)
check_frame(1700, 519, 800, 1077, 50, "Lockup - name at x519,y800")

# g1780 - Thread death (local 1780)
check_frame(1780, 519, 800, 1077, 50, "Lockup - Thread dying at x519,y800")
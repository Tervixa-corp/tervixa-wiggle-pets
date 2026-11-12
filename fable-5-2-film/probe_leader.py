import os
from PIL import Image

img = Image.open(r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills\g0140.png").convert("RGB")
print("size", img.size)
# sample a vertical strip at x560 from y300 to y430
for y in [300, 316, 340, 360, 380, 400, 412, 420, 430]:
    print(f"x560 y{y}: {img.getpixel((560, y))}")
# sample horizontal strip at y412 x540-580 (where vertical meets horizontal)
for x in [540, 550, 560, 570, 580]:
    print(f"x{x} y412: {img.getpixel((x, 412))}")
# check if ANY bright pixel near x560 column
brightest = []
for y in range(300, 430):
    p = img.getpixel((560, y))
    b = sum(p)/3
    brightest.append((b, y, p))
brightest.sort(reverse=True)
print("top 5 brightest on x560 y300-430:", brightest[:5])

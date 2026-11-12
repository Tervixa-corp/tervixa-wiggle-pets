import os
from PIL import Image

base = r"C:\Users\Remya\Documents\Default Project\fable-5-2-film\out\stills"

def scan(f, x0, y0, x1, y1, label, target=None, tol=45):
    img = Image.open(os.path.join(base, f"g{f:04d}.png")).convert("RGB")
    best = []
    for y in range(y0, y1):
        for x in range(x0, x1):
            p = img.getpixel((x, y))
            b = sum(p) / 3
            if target:
                d = sum(abs(p[i] - target[i]) for i in range(3))
                if d <= tol:
                    best.append((b, x, y, p))
            else:
                best.append((b, x, y, p))
    best.sort(reverse=True)
    print(f"\n=== g{f} {label} ===")
    print(f"  brightest 5: {[(b, x, y, p) for b, x, y, p in best[:5]]}")
    print(f"  near-target hits: {len(best)}")

# g0520 - chart: hero bar should be clay, well lit
scan(520, 420, 378, 1580, 54, "hero bar region", (201, 100, 66), 45)

# g0900 - Robo: hero bar clay, multiple bone
scan(900, 420, 556, 1580, 62, "Robo hero bar", (201, 100, 66), 45)
scan(900, 1310, 690, 1580, 840, "Robo multiple", (242, 239, 233), 35)

# g0960 - Fermat hold: dot cluster
scan(960, 260, 536, 1100, 936, "Fermat dots", (242, 239, 233), 40)

# g1000 - Fermat payoff: clock clay, words bone
scan(1000, 1130, 500, 1226, 600, "Fermat clock", (201, 100, 66), 45)
scan(1000, 1130, 330, 1710, 390, "Fermat words", (242, 239, 233), 35)

# g1300 - Economics: -20% clay
scan(1300, 1310, 630, 1520, 800, "Economics -20%", (201, 100, 66), 45)

# g1700 - Lockup: wordmark bone
scan(1700, 519, 606, 1077, 792, "Lockup wordmark", (242, 239, 233), 35)

# g1780 - Lockup: Thread dying, wordmark still bone
scan(1780, 519, 606, 1077, 792, "Lockup wordmark", (242, 239, 233), 35)
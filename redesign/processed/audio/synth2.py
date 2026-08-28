exec(open('common.py').read())
import numpy as np

def bell(freq, dur, decay=6.0, partials=((1,1),(2.76,0.4),(5.4,0.18))):
    tt = t(dur); s = np.zeros(tt.size)
    for mult,amp in partials:
        s += amp*np.sin(2*np.pi*freq*mult*tt)*np.exp(-decay*tt/ (1+mult*0.2))
    return s

def thud(freq, dur, decay=14.0):
    tt = t(dur)
    f = freq*np.exp(-3*tt)
    ph = np.cumsum(2*np.pi*f/SR)
    return np.sin(ph)*np.exp(-decay*tt)

def silence(dur): return np.zeros(int(SR*dur))

def pan(sig, position):
    # position -1 (left) .. +1 (right)
    l = sig*np.sqrt((1-position)/2); r = sig*np.sqrt((1+position)/2)
    return l, r

# ---------- 2. SEASON BOUNDARY MARKER ----------
seq = np.concatenate([silence(0.4), thud(90,1.2)*0.9 + bell(330,1.2,decay=9)*0.25, silence(0.6)])
write_wav('04-season-boundary.wav', fade(seq,0.01))

# ---------- 3. EVENT TIERS ----------
minor    = bell(880, 0.5, decay=12, partials=((1,1),(2.1,0.25)))*0.5
notable  = np.concatenate([bell(660,1.1,decay=5), silence(0.1)]) 
notable += np.concatenate([silence(0.06), bell(990,1.15,decay=5)*0.5])[:notable.size]
epochal  = np.zeros(int(SR*3.2))
ep_t = t(3.2)
for f,a,d in [(110,1.0,1.2),(165,0.6,1.6),(220,0.45,2.2),(330,0.3,2.6)]:
    epochal += a*np.sin(2*np.pi*f*ep_t)*np.exp(-ep_t/d)
epochal += 0.5*thud(70, 3.2, decay=1.6)
epochal = lowpass(epochal, 2200)

tiers = np.concatenate([silence(0.3), minor, silence(0.9),
                        notable, silence(0.9),
                        epochal, silence(0.4)])
write_wav('05-event-tiers.wav', fade(tiers,0.01))

# ---------- 4. OFF-SCREEN SPATIAL CUE ----------
def offscreen(sig, position, distance):
    # distance 0..1 -> duller + quieter
    s = lowpass(sig, 6000 - distance*5200)*(1-0.6*distance)
    return pan(s, position)

parts_l = []; parts_r = []
for pos, dist, label in [(-0.9,0.8,'far left'), (0.0,0.0,'centre/near'), (0.9,0.5,'right mid')]:
    l,r = offscreen(bell(740,0.9,decay=8), pos, dist)
    parts_l += [silence(0.5), l]; parts_r += [silence(0.5), r]
write_wav('06-offscreen-cue.wav', fade(np.concatenate(parts_l),0.01), fade(np.concatenate(parts_r),0.01))

# ---------- 5. FULL MIX DEMO ----------
base = drone(16, 62, warmth=0.35, instability=0.25, openness=0.5)*0.55
mix_l = base.copy(); mix_r = base.copy()
def place(sig, at, position=0.0, dist=0.0, gain=1.0):
    l,r = offscreen(sig*gain, position, dist)
    i = int(SR*at)
    n = min(l.size, mix_l.size-i)
    mix_l[i:i+n] += l[:n]; mix_r[i:i+n] += r[:n]

place(thud(90,1.2)*0.9 + bell(330,1.2,decay=9)*0.25, 1.0, 0.0, 0.0, 0.8)   # season
place(bell(880,0.5,decay=12), 3.4, -0.7, 0.6, 0.45)                         # minor offscreen
place(bell(880,0.5,decay=12), 5.1,  0.5, 0.3, 0.45)                         # minor
place(thud(90,1.2)*0.9 + bell(330,1.2,decay=9)*0.25, 7.0, 0.0, 0.0, 0.8)   # season
place(notable, 8.6, 0.2, 0.1, 0.7)                                          # notable
place(epochal, 11.5, 0.0, 0.0, 0.85)                                        # epochal
write_wav('07-full-mix-demo.wav', fade(mix_l,0.5), fade(mix_r,0.5))

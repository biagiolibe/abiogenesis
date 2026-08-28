import numpy as np, wave, struct

SR = 44100

def write_wav(name, left, right=None):
    if right is None: right = left
    m = max(np.max(np.abs(left)), np.max(np.abs(right)), 1e-9)
    left = left/m*0.85; right = right/m*0.85
    inter = np.empty(left.size*2)
    inter[0::2] = left; inter[1::2] = right
    data = (inter*32767).astype(np.int16)
    with wave.open(name,'w') as w:
        w.setnchannels(2); w.setsampwidth(2); w.setframerate(SR)
        w.writeframes(data.tobytes())
    print("wrote", name, round(left.size/SR,1),"s")

def t(dur): return np.linspace(0, dur, int(SR*dur), endpoint=False)

def lowpass(x, cutoff):
    # simple one-pole
    a = np.exp(-2*np.pi*cutoff/SR)
    y = np.empty_like(x); acc = 0.0
    for i in range(x.size):
        acc = a*acc + (1-a)*x[i]; y[i] = acc
    return y

def fade(x, f=0.02):
    n = int(SR*f); e = np.ones(x.size)
    e[:n] = np.linspace(0,1,n); e[-n:] = np.linspace(1,0,n)
    return x*e


def drone(dur, base, warmth, instability, openness):
    tt = t(dur)
    # slow detuned oscillators
    sig = np.zeros(tt.size)
    for k,(mult,amp) in enumerate([(1,1.0),(1.5,0.4),(2.0,0.25),(3.0,0.12)]):
        det = 1 + 0.002*np.sin(2*np.pi*(0.05+0.02*k)*tt)
        sig += amp*np.sin(2*np.pi*base*mult*det*tt)
    # instability = toxicity: slow wobbling amplitude + slight noise
    wob = 1 + instability*0.5*np.sin(2*np.pi*0.13*tt + 1.0)
    noise = instability*0.05*np.random.RandomState(7).randn(tt.size)
    sig = sig*wob + noise
    # openness = light: brighter cutoff
    cutoff = 180 + openness*900
    sig = lowpass(sig, cutoff)
    # warmth = temperature: adds a soft octave-below body
    sig += warmth*0.35*np.sin(2*np.pi*base*0.5*tt)
    # very slow breathing envelope
    sig *= (0.75 + 0.25*np.sin(2*np.pi*0.07*tt))
    return fade(sig, 1.2)


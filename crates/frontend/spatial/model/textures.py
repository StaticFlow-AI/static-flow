"""Bake seamless equirectangular planet maps and tiled laboratory materials."""
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw

out = Path(__file__).resolve().parent.parent / "art"
out.mkdir(exist_ok=True)
rng = np.random.default_rng(1048596)
table = rng.random((67, 67, 67)).astype(np.float32)

def noise(x,y,z):
    base = [np.floor(v).astype(int) for v in [x,y,z]]
    f = [v-b for v,b in zip((x,y,z),base)]
    sx,sy,sz = [q*q*(3-2*q) for q in f]
    def at(i,j,k): return table[(base[0]+i)%67,(base[1]+j)%67,(base[2]+k)%67]
    def mix(a,b,v): return a+(b-a)*v
    return mix(mix(mix(at(0,0,0),at(1,0,0),sx),mix(at(0,1,0),at(1,1,0),sx),sy),
               mix(mix(at(0,0,1),at(1,0,1),sx),mix(at(0,1,1),at(1,1,1),sx),sy),sz)
def fb(x,y,z,freq=3):
    return sum(noise(x*freq*2**i+5,y*freq*2**i+13,z*freq*2**i+27)*(.52**i) for i in range(6))/2.04
def palette(v,keys):
    return np.stack([np.interp(v,[p[0] for p in keys],[p[1][c] for p in keys]) for c in range(3)],axis=-1)
def save(name,pixels):
    Image.fromarray(np.uint8(np.clip(pixels,0,255))).save(out/(name+'.webp'),quality=88,method=6)
w,h=768,384
lat,lon=np.mgrid[0:h,0:w];lat=(.5-lat/(h-1))*np.pi;lon=lon/w*np.pi*2
x=np.cos(lat)*np.cos(lon);y=np.sin(lat);z=np.cos(lat)*np.sin(lon)
for i,name in enumerate(['archive','music','gallery','studio','tools','search','sun']):
    a=fb(x+i*2,y+1,z,2.7); fine=fb(x+13,y+i,z+4,12)
    if name=='archive':
        col=palette(a,[(.2,(13,38,71)),(.46,(28,74,110)),(.49,(66,128,128)),(.50,(169,161,109)),(.55,(94,126,91)),(.63,(119,109,86)),(.77,(194,187,161))])
        clouds=np.clip((fb(x+2*a,y*.9,z+3*a,7)-.51)*4,0,.85)[...,None]
        ice=np.clip((abs(y)-.86+fine*.09)*6,0,.95)[...,None]
        col=col*(1-clouds)+np.array([226,231,222])*clouds;col=col*(1-ice)+[228,235,228]*ice
    elif name in ['music','tools']:
        bands=.5+.29*np.sin(y*51+a*9)+.12*np.sin(y*103+fine*9)+.13*(a-.5)
        keys=[(0,(96,57,54)),(.25,(162,98,68)),(.40,(216,164,114)),(.57,(235,210,170)),(.70,(174,128,106)),(1,(242,221,186))] if name=='music' else [(0,(39,89,102)),(.4,(84,146,153)),(.6,(132,191,180)),(1,(176,220,194))]
        col=palette(bands,keys)
    elif name=='sun':
        col=palette(.67*a+.33*fine,[(.18,(211,74,12)),(.38,(247,137,32)),(.52,(255,205,90)),(.68,(255,239,156)),(.85,(255,254,214))])
    else:
        colors={'gallery':[(46,37,73),(113,82,138),(207,168,190)],'studio':[(87,48,41),(173,93,55),(223,159,99)],'search':[(48,88,121),(119,175,201),(214,233,231)]}[name]
        terrain=np.clip(a*.67+fine*.33+np.sin(a*41)*.038,0,1)
        col=palette(terrain,[(.20,colors[0]),(.46,colors[1]),(.73,colors[2])])
        for crater in range(22):
            d=rng.normal(size=3);d/=np.linalg.norm(d);r=rng.uniform(.012,.095)
            dist=np.maximum(0,2-2*(d[0]*x+d[1]*y+d[2]*z))**.5
            col*= (1+.14*np.exp(-((dist-r)/(r*.13))**2)-.15*np.exp(-(dist/(r*.75))**4))[...,None]
    save(name,col*(.93+.14*fine[...,None]))
n=512
yy,xx=np.mgrid[0:n,0:n]; grain=rng.normal(0,3,(n,n))
p=fb(xx/n,yy/n,xx*0+4.,5)
save('plaster',np.repeat((221+22*(p-.5)+grain)[...,None],3,axis=2))
floor_img=np.zeros((n,n,3))
for j in range(8):
    row=yy//64==j;shift=(j*141)%256
    board=rng.uniform(.87,1.05,(3,))[((xx+shift)//256)%3]
    line=(yy%64<=2)|((xx+shift)%256<=2)
    wave=6*np.sin(yy*1.4+7*noise(xx/n*3,yy/n*13,xx*0+2.))+4*np.sin(yy*.7+xx*.01)
    value=(225+wave+grain)*board-(line*27)
    for k in range(3):floor_img[:,:,k][row]=value[row]*[1,.96,.89][k]
save('parquet',floor_img)
city=Image.new('RGB',(256,256),(172,171,160));d=ImageDraw.Draw(city)
for j in range(7):
    for i in range(5):
        x0=7+i*51;y0=6+j*36;c=int(rng.integers(65,135));d.rectangle((x0,y0,x0+32,y0+22),fill=(78,80,79))
        d.rectangle((x0+2,y0+2,x0+30,y0+19),fill=(c,c+8,c+9) if rng.random()>.20 else (199,181,136))
        d.line((x0+17,y0,x0+17,y0+21),fill=(120,123,116),width=2)
        d.rectangle((x0-2,y0+23,x0+35,y0+25),fill=(137,138,129))
city.save(out/'facade.webp',quality=88)

# A seamless deep-sky panorama. The dust lane is a tilted great circle.
height, width = 512, 1024
iy, ix = np.mgrid[:height, :width]
lat = (.5 - iy/(height-1))*np.pi
lon = ix/width*2*np.pi
x, y, z = np.cos(lat)*np.cos(lon), np.sin(lat), np.cos(lat)*np.sin(lon)
warp = fb(x+5,y+23,z+11,3)
plane = -.28*x + .84*y - .42*z + .17*(warp-.5)
density = np.exp(-np.square(plane/.23))
cloud = np.clip((fb(x+1.8*warp,y+8,z+2,6)-.35)*3.8,0,1.45)
rift = np.clip(np.abs(plane-.026-.09*(fb(x+2,y,z+5,10)-.5))/.070,.18,1)
filament = density * cloud**1.7 * rift
hue = np.clip(.4+x*.55+2*(warp-.5),0,1)[...,None]
color = (1-hue)*np.array([40,62,93]) + hue*np.array([91,50,87])
bg = np.array([5,9,21])+filament[...,None]*color
bg += (density*(cloud+.2)*.10)[...,None]*np.array([49,66,91])
save('deep-sky',bg)

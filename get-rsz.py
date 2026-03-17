import urllib.request
import datetime

print("Downloading file... ", end = '')
urllib.request.urlretrieve('https://raw.githubusercontent.com/dtlnor/RE_RSZ/refs/heads/MHWilds/rszmhwilds.json', 'rszmhwilds.json')

with open('rszmhwilds.json.version', 'w') as f:
    f.write(datetime.datetime.now().isoformat())

print("done!")
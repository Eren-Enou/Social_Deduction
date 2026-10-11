import hashlib,json,shutil,subprocess
from pathlib import Path

root=Path.cwd()
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
assert head=='f13bb0998a3bf6f3e63299e276015becf1452aa5'
assert not subprocess.check_output(['git','status','--porcelain'])
backup=root.parent/'SocialDeduction-publication-backup-f13bb09-20261010'
backup.mkdir(exist_ok=False)
def digest_file(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        while b:=f.read(1048576):h.update(b)
    return h.hexdigest()
subprocess.run(['git','bundle','create',str(backup/'original.bundle'),'--all'],check=True)
subprocess.run(['git','bundle','verify',str(backup/'original.bundle')],check=True)
subprocess.run(['git','clone','--mirror',str(backup/'original.bundle'),str(backup/'verified.git')],check=True)
subprocess.run(['git','--git-dir='+str(backup/'verified.git'),'fsck','--full'],check=True)
assert subprocess.check_output(['git','--git-dir='+str(backup/'verified.git'),'rev-parse','refs/heads/codex/experiment-012'],text=True).strip()==head
files=[]
(backup/'evidence').mkdir()
for source in sorted((root/'evidence').glob('*.json')):
    dest=backup/'evidence'/source.name
    with source.open('rb') as src,dest.open('xb') as dst:shutil.copyfileobj(src,dst)
    sha=digest_file(source)
    assert sha==digest_file(dest) and source.stat().st_size==dest.stat().st_size
    files.append({'path':'evidence/'+source.name,'bytes':source.stat().st_size,'sha256':sha})
objects=subprocess.check_output(['git','cat-file','--batch-all-objects','--batch-check=%(objectname) %(objecttype) %(objectsize)'],text=True).splitlines()
paths={}
for line in subprocess.check_output(['git','rev-list','--objects','--all'],text=True).splitlines():
    oid,_,path=line.partition(' ')
    if path:paths.setdefault(oid,[]).append(path)
large=[]
for line in objects:
    oid,kind,size=line.split()
    if kind!='blob' or int(size)<=100*1024*1024:continue
    p=subprocess.Popen(['git','cat-file','blob',oid],stdout=subprocess.PIPE)
    h=hashlib.sha256();count=0
    while chunk:=p.stdout.read(1048576):h.update(chunk);count+=len(chunk)
    assert p.wait()==0 and count==int(size)
    large.append({'blob':oid,'bytes':count,'sha256':h.hexdigest(),'paths':paths.get(oid,[]),'reachable':oid in paths})
record={'original_head':head,'original_branch':'codex/experiment-012','bundle_bytes':(backup/'original.bundle').stat().st_size,'bundle_sha256':digest_file(backup/'original.bundle'),'artifacts':files,'oversized_blobs':large,'all_git_objects_scanned':len(objects)}
(backup/'backup-manifest.json').write_text(json.dumps(record,indent=2)+'\n')
(root/'target/publication-backup-record.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'verified_backup':str(backup),'artifacts':len(files),'oversized_blobs':large,'bundle_sha256':record['bundle_sha256']},indent=2))

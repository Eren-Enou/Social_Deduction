import hashlib,json,subprocess
from pathlib import Path

source=Path.cwd()
backup=source.parent/'SocialDeduction-publication-backup-f13bb09-20261010'
record=json.loads((backup/'backup-manifest.json').read_text())
assert record['original_head']=='f13bb0998a3bf6f3e63299e276015becf1452aa5'
dest=source.parent/'SocialDeduction-publishing-012-v2'
dest.mkdir(exist_ok=False)
subprocess.run(['git','init',str(dest)],check=True)
old_marks=backup/'original-export-v2.marks'
new_marks=backup/'publishing-import-v2.marks'
export=subprocess.Popen(['git','fast-export','--show-original-ids','--export-marks='+str(old_marks),'codex/experiment-012'],stdout=subprocess.PIPE)
imp=subprocess.Popen(['git','fast-import','--quiet','--export-marks='+str(new_marks)],cwd=dest,stdin=subprocess.PIPE)
buffer=None;mark=None;oid=None;dropped=set();stripped=[];skip_separator=False
while line:=export.stdout.readline():
    if skip_separator:
        assert line==b'\n'
        skip_separator=False
        continue
    if line==b'blob\n':
        buffer=[line];mark=None;oid=None;continue
    if buffer is not None and not line.startswith(b'data '):
        buffer.append(line)
        if line.startswith(b'mark '):mark=line.split()[1]
        if line.startswith(b'original-oid '):oid=line.split()[1].decode()
        continue
    if line.startswith(b'data '):
        size=int(line.split()[1]);drop=buffer is not None and size>100*1024*1024
        h=hashlib.sha256()
        if not drop:
            if buffer is not None:imp.stdin.write(b''.join(buffer))
            imp.stdin.write(line)
        remaining=size
        while remaining:
            chunk=export.stdout.read(min(1048576,remaining))
            assert chunk
            remaining-=len(chunk)
            if drop:h.update(chunk)
            else:imp.stdin.write(chunk)
        if drop:
            assert mark and oid
            item=next(x for x in record['oversized_blobs'] if x['blob']==oid)
            assert item['bytes']==size and item['sha256']==h.hexdigest()
            dropped.add(mark);stripped.append(item)
            skip_separator=True
        buffer=None
        continue
    if line.startswith(b'M '):
        parts=line.rstrip(b'\n').split(b' ',3)
        if parts[2] in dropped:line=b'D '+parts[3]+b'\n'
    imp.stdin.write(line)
imp.stdin.close()
assert export.wait()==0 and imp.wait()==0
subprocess.run(['git','branch','-m','codex/experiment-012','codex/publish-experiment-012'],cwd=dest,check=True)
subprocess.run(['git','symbolic-ref','HEAD','refs/heads/codex/publish-experiment-012'],cwd=dest,check=True)
subprocess.run(['git','reset','--hard','HEAD'],cwd=dest,check=True)
subprocess.run(['git','remote','add','origin','https://github.com/Eren-Enou/Social_Deduction.git'],cwd=dest,check=True)
subprocess.run(['git','fsck','--full'],cwd=dest,check=True)
def marks(path):return dict(line.split() for line in path.read_text().splitlines())
old=marks(old_marks);new=marks(new_marks)
commits=set(subprocess.check_output(['git','rev-list','codex/experiment-012'],text=True).splitlines())
mapping=[{'original':oid,'publishing':new[m]} for m,oid in old.items() if oid in commits]
assert len(mapping)==len(commits)
remove_paths={p for item in stripped for p in item['paths']}
def tree(repo,commit):
    lines=subprocess.check_output(['git','ls-tree','-r',commit],cwd=repo,text=True).splitlines()
    return {path:meta for meta,path in (line.split('\t',1) for line in lines)}
for row in mapping:
    original=tree(source,row['original'])
    sanitized=tree(dest,row['publishing'])
    for path in remove_paths:original.pop(path,None)
    assert original==sanitized,row
    fmt='%an%n%ae%n%at%n%cn%n%ce%n%ct%n%B'
    assert subprocess.check_output(['git','show','-s','--format='+fmt,row['original']],cwd=source)==subprocess.check_output(['git','show','-s','--format='+fmt,row['publishing']],cwd=dest),row
objects=subprocess.check_output(['git','cat-file','--batch-all-objects','--batch-check=%(objectname) %(objecttype) %(objectsize)'],cwd=dest,text=True).splitlines()
assert not [x for x in objects if x.split()[1]=='blob' and int(x.split()[2])>100*1024*1024]
result={'original_head':record['original_head'],'sanitized_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=dest,text=True).strip(),'publishing_branch':'codex/publish-experiment-012','stripped_blobs':stripped,'commit_mapping':mapping,'verified_all_commit_trees_except_removed_artifacts':True,'verified_commit_authors_dates_messages':True,'no_oversized_blobs':True}
(backup/'migration-history.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'migration_clone':str(dest),'mapped_commits':len(mapping),'stripped_blobs':len(stripped),'sanitized_head':result['sanitized_head']},indent=2))

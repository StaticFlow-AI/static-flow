const path=require('node:path');
const {promisify}=require('node:util');
const exec=promisify(require('node:child_process').execFile);
function createRotator({env=process.env,run=exec,notify=console.log}={}){
  let completed=0;
  const enabled=env.KIRO_CLASH_ROTATE!=='0';
  const invoke=async command=>{
    const {stdout}=await run(env.PYTHON||'python3',[path.join(__dirname,'kiro_clash_rotate.py'),command,env.KIRO_CLASH_GROUP||'auto'],{timeout:30000});
    return JSON.parse(stdout);
  };
  return {
    async check(){if(enabled)await invoke('check');},
    completed(){if(enabled)completed++;},
    async beforeAccount(){
      if(!enabled||completed<2)return;
      const result=await invoke('next');
      completed=0;
      notify('CLASH_ROTATED '+JSON.stringify(result));
    },
  };
}
module.exports={createRotator};

import assert from 'node:assert/strict';
import {spawn,execFile} from 'node:child_process';
import fs from 'node:fs/promises';
import {promisify} from 'node:util';
const execute=promisify(execFile);
const root='/home/apnex/taceng/agp';
const evidence='/home/apnex/taceng/cli/target/agp-integration';
const binary=`${evidence}/install/bin/agp`;
const node=spawn(process.execPath,['examples/loopback-star/example.mjs','--persist'],{cwd:root,env:{...process.env,AGP_LOOPBACK_HUB_MANAGEMENT_PORT:'0',AGP_LOOPBACK_ALPHA_MANAGEMENT_PORT:'0',AGP_LOOPBACK_BETA_MANAGEMENT_PORT:'0'},stdio:['ignore','pipe','pipe']});
let nodeOutput='',nodeError='';
node.stderr.on('data',bytes=>nodeError+=bytes);
const exited=new Promise(resolve=>node.once('exit',(code,signal)=>resolve({code,signal})));
try {
 const ready=await new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>reject(Error('Example readiness deadline: '+nodeError)),15000);
  node.stdout.on('data',bytes=>{
   nodeOutput+=bytes;
   const line=nodeOutput.split('\n').find(line=>line.startsWith('AGP_LOOPBACK_TOPOLOGY_READY '));
   if(line){clearTimeout(timer);resolve(JSON.parse(line.slice('AGP_LOOPBACK_TOPOLOGY_READY '.length)));}
  });
  node.once('exit',code=>{clearTimeout(timer);reject(Error('Example exited '+code+': '+nodeError));});
 });
 const url=ready.nodes.hub.managementUrl;
 const cases=[['--help'],['tree'],['connections','show'],['routes','ls'],['health','show'],['resources','show'],['counters','show'],['snapshot','show','--json']];
 const report=[];
 for(const args of cases){
  const result=await execute(binary,args,{cwd:root,env:{...process.env,AGP_MANAGEMENT_URL:url,PATH:''},timeout:15000,maxBuffer:8*1024*1024});
  assert.equal(result.stderr,'');
  assert.ok(result.stdout.length>0);
  if(args.includes('--json')){const value=JSON.parse(result.stdout);assert.equal(value.kind,'OperationsSnapshot');assert.equal(value.meta.nodeId,'hub');}
  await fs.writeFile(`${evidence}/guide-${args[0].replaceAll('-','')}.txt`,result.stdout);
  report.push({arguments:args,exit:0,bytes:Buffer.byteLength(result.stdout)});
 }
 const q=value=>`'${value.replaceAll("'", "'\\''")}'`;
 const terminal=spawn('/usr/bin/script',['-qefc',`stty cols 120 rows 30; exec ${q(binary)} --url ${q(url)}`,'/dev/null'],{cwd:root,env:{...process.env,TERM:'xterm-256color',AGP_MANAGEMENT_URL:''},stdio:['pipe','pipe','pipe'],detached:true});
 let transcript='',pending='',stage=0;
 const actions=[['agp [/]','connections\r'],['agp [connections]','show\r'],['SESSION_ID',''],['agp [connections]','?\r'],['agp / connections',''],['agp [connections]','up\r'],['agp [/]','tree\r'],['agp [/]','exit\r']];
 const result=await new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>{process.kill(-terminal.pid,'SIGTERM');reject(Error('Terminal deadline at stage '+stage));},30000);
  terminal.stdout.on('data',bytes=>{
   const text=bytes.toString();transcript+=text;pending+=text;
   for(const match of text.matchAll(/\x1b\[6n/g))terminal.stdin.write('\x1b[1;1R');
   while(stage<actions.length){const [needle,input]=actions[stage];const index=pending.indexOf(needle);if(index<0)break;pending=pending.slice(index+needle.length);if(input)terminal.stdin.write(input);stage++;}
  });
  terminal.once('error',error=>{clearTimeout(timer);reject(error);});
  terminal.once('exit',(code,signal)=>{clearTimeout(timer);resolve({code,signal,stage});});
 });
 await fs.writeFile(`${evidence}/operator-terminal.raw`,transcript);
 assert.equal(result.code,0,transcript);assert.equal(result.stage,actions.length,transcript);
 assert.ok(transcript.includes('Established'));
 await fs.writeFile(`${evidence}/operator-guide.json`,JSON.stringify({node:'hub',managementUrl:url,cases:report,terminal:result},null,2)+'\n');
 console.log(JSON.stringify({cases:report.length,terminal:result,node:'hub'}));
} finally {
 node.kill('SIGTERM');
 const stopped=await exited;
 await fs.writeFile(`${evidence}/guide-example.stdout`,nodeOutput);
 await fs.writeFile(`${evidence}/guide-example.stderr`,nodeError);
 assert.equal(stopped.code,0,JSON.stringify(stopped));
}

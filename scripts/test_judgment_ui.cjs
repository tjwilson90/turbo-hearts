// DOM simulation only; no browser or live application data. Requires jsdom for tests.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {JSDOM} = require('jsdom');
const root = path.resolve(__dirname,'..');
const cases = fs.readFileSync(path.join(root,'server/judgment-cases.jsonl'),'utf8').trim().split('\n').map(JSON.parse);
const html = fs.readFileSync(path.join(__dirname,'judgment_ui/index.html'),'utf8');
const code = fs.readFileSync(path.join(__dirname,'judgment_ui/app.js'),'utf8');
async function settled(check) {
  for(let i=0;i<100;i++) { if(check())return; await new Promise(setImmediate); }
  assert.fail('UI did not settle');
}
async function setup(kind) {
  const dom = new JSDOM(html,{url:'https://hearts.example/judge',runScripts:'outside-only'});
  const w=dom.window; w.structuredClone=structuredClone; w.confirm=()=>true;
  const question=structuredClone(cases.find(c=>c.view.kind===kind));
  let saved=null, revision=0; const requests=[];
  w.fetch=async(url,options)=>{
    const body=JSON.parse(options.body); requests.push({url,options,body});
    assert.equal(options.headers['X-Judgment-Request'],'1');
    assert.equal(options.credentials,'same-origin');
    assert.equal(options.method,'POST');
    assert.equal(body.reviewer,undefined);
    if(url==='/judge/api/review') { saved=body;revision++;return {ok:true,json:async()=>({saved:true,revision})}; }
    assert.equal(url,'/judge/api/state');
    return {ok:true,json:async()=>({
      case:!saved||body.case?{id:question.id,view:question.view,revision,annotation:saved}:null,
      progress:{all:{pending:saved?0:1,answered:saved?.status==='answered'?1:0,skipped:saved?.status==='skipped'?1:0}},
      recent:saved?[{case_id:question.id,kind,direction:question.view.direction,status:saved.status,created_at:'2026-11-08T12:00:00Z'}]:[]
    })};
  };
  w.eval(code+'\nwindow.renderFixture=showQuestion;');
  await settled(()=>w.document.querySelectorAll('#hand button').length>0);
  return {dom,w,question,requests,$:id=>w.document.getElementById(id),getSaved:()=>saved};
}

for(const kind of ['pass','charge','play'])test(`${kind}: choice, save, empty queue, edit, skip`,async()=>{
  const h=await setup(kind); const {$,w}=h;
  try {
    assert.equal($('reviewer-label').hidden,true);
    assert.equal($('lobby-link').hidden,false);
    assert.equal($('save').disabled,true);
    if(kind==='charge') $('decline').click();
    else {
      const count=kind==='pass'?3:1;
      for(let i=0;i<count;i++) w.document.querySelector('#hand button:not([disabled])[aria-pressed=false]').click();
    }
    assert.equal($('save').disabled,false);
    if(kind==='pass') {
      $('add-alternative').click();
      for(let i=0;i<3;i++) Array.from(w.document.querySelectorAll('#hand button:not([disabled])[aria-pressed=false]')).at(-1).click();
      $('keep-alternative').click();
    }
    $('confidence').value='high';$('note').value='Keep control; compare the long suit.';
    $('save').click();
    await settled(()=>$('empty').hidden===false);
    assert.equal(h.getSaved().action.length,kind==='pass'?3:kind==='play'?1:0);
    assert.equal(h.getSaved().alternatives.length,kind==='pass'?1:0);
    assert.equal(h.getSaved().confidence,'high');
    assert.equal($('workspace').hidden,true);
    assert.equal($('recent').closest('[hidden]'),null,'Recent judgments must remain available with an empty queue');
    $('recent').querySelector('button').click();
    await settled(()=>$('workspace').hidden===false);
    assert.match($('message').textContent,/Editing/);
    assert.equal($('save').disabled,false);
    assert.equal($('note').value,'Keep control; compare the long suit.');
    $('skip').click();await settled(()=>h.getSaved().status==='skipped' && $('workspace').hidden);
    assert.equal(h.getSaved().expected_revision,1);
  } finally {h.dom.window.close();}
});

test('all starter positions render without missing history or card fields',async()=>{
  const h=await setup('play');
  try {
    for(const c of cases) {
      h.w.renderFixture({id:c.id,view:c.view,revision:0,annotation:null});
      assert.equal(h.w.document.querySelectorAll('#hand button').length,c.view.hand.length);
      assert.equal(h.w.document.querySelectorAll('#hand button:not([disabled])').length,c.view.legal_cards.length);
      assert.ok(!h.$('players').textContent.includes('undefined'));
      assert.ok(!h.$('preplay-history').textContent.includes('undefined'));
      assert.equal(h.$('save').disabled,true);
    }
  } finally {h.dom.window.close();}
});

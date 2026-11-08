'use strict';
const deployed = location.pathname === '/judge' || location.pathname.startsWith('/judge/');
const $ = id => document.getElementById(id);
const seats = ['north','east','south','west'];
const symbols = {S:'♠',H:'♥',D:'♦',C:'♣'};
const names = {avoid_points:'Avoid points',shoot_moon:'Shoot the moon',stop_moon:'Stop a moon',win_jack:'Win J♦',manage_ten:'Manage 10♣',create_void:'Create a void',keep_control:'Keep control / a stopper',shed_low_cards:'Shed low cards',give_points:'Give someone points',other:'Another plan'};
let current = null, primary = [], alternatives = [], working = [], mode = 'preferred';
let decided = false, altDecided = false, busy = false, started = 0, requestNumber = 0;
const cap = s => s ? s.charAt(0).toUpperCase()+s.slice(1) : '';
const cardText = c => (c[0]==='T'?'10':c[0])+symbols[c[1]];
const actionText = a => a.length ? a.map(cardText).join(' ') : 'No charge';
const same = (a,b) => [...a].sort().join(',') === [...b].sort().join(',');
function node(tag, text, cls) { const e=document.createElement(tag); if(text!==undefined)e.textContent=text; if(cls)e.className=cls;return e; }
function message(text,error=false) { $('message').textContent=text; $('message').className=error?'error':''; }
let cleanDraft = '';
function draft() { return JSON.stringify([current?.id, primary, alternatives, working, mode, decided, $('confidence').value, $('runner').value, $('note').value, $('recognized').checked, [...$('intents').querySelectorAll('input:checked')].map(e=>e.value)]); }
function dirty() { return current && cleanDraft !== draft(); }
function navigate(id) { if(!busy && (!dirty() || confirm('Discard your unsaved changes?'))) { message(''); load(id); } }
function reviewer() { return $('reviewer').value.trim() || 'local'; }
function face(card, button=false) {
  const e=node(button?'button':'span',undefined,'card'+('HD'.includes(card[1])?' red':''));
  e.append(node('span',card[0]==='T'?'10':card[0]),node('span',symbols[card[1]],'suit'));
  e.setAttribute('aria-label',cardText(card)); return e;
}
async function api(path, options) {
  const response=await fetch((deployed?'/judge':'')+path,{...options,credentials:'same-origin',headers:{...(options?.headers||{}),...(deployed?{'X-Judgment-Request':'1'}:{})}});
  const result=await response.json().catch(()=>({error:'Unable to load the decision lab. Please sign in again.'}));
  if(deployed)$('signin-link').hidden=response.status!==401;
  if(!response.ok)throw new Error(result.error || 'Request failed.'); return result;
}
function renderPlayers(v) {
  $('players').replaceChildren();
  for(const p of v.players) {
    const rel=(seats.indexOf(p.seat)-seats.indexOf(v.seat)+4)%4;
    const e=node('div',undefined,'player compass-seat'+(rel===0?' you':''));
    e.dataset.seat=p.seat;
    const leader=v.kind==='play'?(v.trick[0]?.seat||v.seat):null;
    e.append(seatHeading(p.seat,leader,rel===0,!v.trick.length),node('small',(p.human?'Human':'Bot')+(rel?` · ${['','left','across','right'][rel]}`:'')));
    const taken=[p.queen?'Q♠':'',p.jack?'J♦':'',p.ten?'10♣':''].filter(Boolean).join(' ');
    e.append(node('div',`${p.hearts} ♥${taken?' · '+taken:''}`,'holdings'));
    e.append(node('small',`Earlier hands: ${p.previous_money>0?'+':''}${p.previous_money}`));
    if(p.charges.length)e.append(node('div','Charged '+p.charges.map(cardText).join(' '),'charges'));
    if(v.kind==='charge' && p.declined)e.append(node('small','Finished current charging turn'));
    if(p.voids.length)e.append(node('small','Void: '+p.voids.map(s=>symbols[s[0]]).join(' ')));
    if(v.kind==='play')e.append(trickCards(v.trick,p.seat));
    $('players').append(e);
  }
}
function seatHeading(seat,leader,you=false,unplayed=false) {
  const heading=node('div',undefined,'seat-heading');
  heading.append(node('strong',cap(seat)+(you?' · you':''),'seat-name'));
  if(seat===leader)heading.append(node('span',unplayed?'Leads':'Led','lead-badge'));
  return heading;
}
function trickCards(plays,seat) {
  const cards=node('div',undefined,'seat-cards');
  plays.forEach((p,i)=>{
    if(p.seat!==seat)return;
    const item=node('div',undefined,'trick-card'),card=face(p.card);
    item.dataset.card=p.card;item.dataset.play=String(i+1);
    if(i===0)card.classList.add('lead-card');
    card.setAttribute('aria-label',`${cardText(p.card)}, play ${i+1}${i===0?', lead':''}`);
    item.append(card,node('span',String(i+1),'play-number'));
    cards.append(item);
  });
  if(!cards.children.length)cards.append(node('span','—','not-played'));
  return cards;
}
function renderTrick(v) {
  $('trick-title').textContent=v.kind==='play'?'Current trick':'Before play';
  $('trick-summary').textContent=v.trick.length?`${v.trick.length} cards played · ${cap(v.trick[0].seat)} led`:
    v.kind==='play'?'You lead the next trick.':'No cards have been played yet.';
}
function renderRemaining(v) {
  $('remaining-panel').hidden=v.kind!=='play';
  $('remaining-cards').replaceChildren();
  if(v.kind!=='play')return;
  // Only the supplied public history and the actor's hand are used. A card on
  // the table is already played, even while an extended trick is incomplete.
  const played=new Set(v.played);
  for(const e of v.history)if(e.type==='play')played.add(e.card);
  for(const p of v.trick)played.add(p.card);
  const own=new Set(v.hand);
  $('remaining-count').textContent=`${52-played.size} unplayed · ${52-played.size-own.size} elsewhere`;
  for(const suit of 'SHDC') {
    const row=node('div',undefined,'remaining-suit'),ranks=node('div',undefined,'remaining-ranks');
    row.append(node('span',symbols[suit],'suit-label'+('HD'.includes(suit)?' red':'')));
    for(const rank of 'AKQJT98765432') {
      const card=rank+suit,status=played.has(card)?'played':own.has(card)?'in-hand':'elsewhere';
      const e=node('span',rank==='T'?'10':rank,'remaining-card '+status+('HD'.includes(suit)?' red':''));
      e.dataset.card=card;
      const label=`${cardText(card)} · ${{played:'played','in-hand':'in your hand',elsewhere:'unplayed elsewhere'}[status]}`;
      e.title=label;e.setAttribute('aria-label',label);ranks.append(e);
    }
    row.append(ranks);$('remaining-cards').append(row);
  }
}
function renderHistory(v) {
  $('preplay-history').replaceChildren();$('trick-history').replaceChildren();
  let plays=[], trickCount=0;
  for(const e of v.history) {
    let text='', own=false;
    switch(e.type) {
      case 'send_pass': text=`You sent ${e.cards.map(cardText).join(' ')}${v.direction==='keeper'?' to the pool':' to '+cap(v.receiver)}.`;own=true;break;
      case 'recv_pass': text=`You received ${e.cards.map(cardText).join(' ')}${v.direction==='keeper'?' from the pool':' from '+cap(v.sender)}.`;own=true;break;
      case 'hidden_send_pass': text=`${cap(e.from)} sent ${e.count} cards.`;break;
      case 'hidden_recv_pass': text=`${cap(e.to)} received ${e.count} cards.`;break;
      case 'charge': text=`${cap(e.seat)} ${e.cards.length?'charged '+e.cards.map(cardText).join(' '):'declined to charge'}.`;break;
      case 'blind_charge': text=`${cap(e.seat)} ${e.count?'blind-charged '+e.count+' card'+(e.count===1?'':'s'):'declined to charge'}.`;break;
      case 'reveal_charges': text='Charges revealed: '+seats.filter(s=>e[s].length).map(s=>cap(s)+' '+e[s].map(cardText).join(' ')).join(' · ');break;
      case 'claim': text=`${cap(e.seat)} claimed with ${e.hand.map(cardText).join(' ')}.`;break;
      case 'accept_claim': text=`${cap(e.acceptor)} accepted ${cap(e.claimer)}’s claim.`;break;
      case 'reject_claim': text=`${cap(e.rejector)} rejected ${cap(e.claimer)}’s claim.`;break;
      case 'play': plays.push(e);break;
      case 'end_trick':
        if(plays.length) {
          const row=node('section',undefined,'history-trick compass');
          const center=node('div',undefined,'table-center');
          center.append(node('strong',`Trick ${++trickCount}`),node('span',`${cap(e.winner)} won`));
          row.setAttribute('aria-label',`Trick ${trickCount}, ${cap(plays[0].seat)} led, ${cap(e.winner)} won`);
          row.append(center);
          for(const seat of seats) {
            const player=node('div',undefined,'compass-seat history-seat'+(seat===e.winner?' winner':''));
            player.dataset.seat=seat;
            player.append(seatHeading(seat,plays[0].seat),trickCards(plays,seat));row.append(player);
          }
          $('trick-history').append(row);plays=[];
        } break;
    }
    if(text)$('preplay-history').append(node('div',text,own?'private':''));
  }
  $('history-count').textContent=trickCount?`${trickCount} completed trick${trickCount===1?'':'s'}`:'';
  if(!$('preplay-history').children.length && !trickCount)$('preplay-history').append(node('p','No passes, charges, or completed tricks yet.','muted'));
}
function selected() { return mode==='preferred'?primary:working; }
function valid(a) { return current && a.length>=current.view.min_cards && a.length<=current.view.max_cards; }
function renderHand() {
  if(!current)return;
  const v=current.view, selection=selected();$('hand').replaceChildren();
  for(const suit of 'SHDC') {
    const cards=v.hand.filter(c=>c[1]===suit).sort((a,b)=>'23456789TJQKA'.indexOf(b[0])-'23456789TJQKA'.indexOf(a[0]));
    if(!cards.length)continue;
    const row=node('div',undefined,'suit-row');row.append(node('span',symbols[suit],'suit-label'+('HD'.includes(suit)?' red':'')));
    for(const c of cards) {
      const e=face(c,true);e.disabled=busy||!v.legal_cards.includes(c);e.setAttribute('aria-pressed',String(selection.includes(c)));
      if(!v.legal_cards.includes(c))e.title='Unavailable for this decision';
      e.addEventListener('click',()=>choose(c));row.append(e);
    }
    $('hand').append(row);
  }
  $('selection-count').textContent=`${selection.length} selected${v.kind==='charge'?'':` / ${v.max_cards}`}`;
  $('choice-label').textContent=mode==='preferred'?'Preferred choice':'Alternative choice';
  const ready=mode==='preferred'?decided:altDecided;
  $('chosen').textContent=ready?actionText(selection):'Choose cards';
  $('decline').hidden=v.kind!=='charge';$('decline').disabled=busy;
  $('save').disabled=busy||mode!=='preferred'||!decided||!valid(primary);
  $('skip').disabled=busy;
  $('add-alternative').hidden=mode!=='preferred';$('add-alternative').disabled=busy||!decided||!valid(primary)||alternatives.length>=20;
  $('keep-alternative').hidden=mode!=='alternative';$('keep-alternative').disabled=busy||!altDecided||!valid(working);
  $('cancel-alternative').hidden=mode!=='alternative';
  $('alternatives').replaceChildren();
  alternatives.forEach((a,i)=>{const e=node('span',undefined,'alternative');e.append(node('span',actionText(a)));const remove=node('button','×');remove.setAttribute('aria-label','Remove alternative '+actionText(a));remove.onclick=()=>{alternatives.splice(i,1);renderHand();};e.append(remove);$('alternatives').append(e);});
  if(mode==='alternative')$('alternatives').prepend(node('span','Preferred: '+actionText(primary),'alternative'));
}
function choose(card) {
  if(busy||!current)return;
  const a=[...selected()];const index=a.indexOf(card);
  if(index>=0)a.splice(index,1);
  else if(current.view.max_cards===1)a.splice(0,a.length,card);
  else if(a.length<current.view.max_cards)a.push(card);
  else {message(`Choose at most ${current.view.max_cards} cards. Deselect one first.`,true);return;}
  if(mode==='preferred'){primary=a;decided=a.length>0;}else{working=a;altDecided=a.length>0;}
  message('');renderHand();
}
function showQuestion(c) {
  current=c;mode='preferred';working=[];altDecided=false;started=performance.now();
  const a=c.annotation;
  primary=a?.action?[...a.action]:[];decided=a?.status==='answered';alternatives=a?.alternatives?structuredClone(a.alternatives):[];
  $('confidence').value=a?.confidence||'';$('runner').value=a?.suspected_runner||'unknown';$('note').value=a?.note||'';$('recognized').checked=a?.recognized||false;
  for(const e of $('intents').querySelectorAll('input'))e.checked=(a?.intents||[]).includes(e.value);
  $('reasoning').open=Boolean(a?.note || a?.intents?.length || (a?.suspected_runner && a.suspected_runner!=='unknown'));
  const v=c.view;
  $('context').textContent=`Hand ${v.hand_number} · ${cap(v.direction)} round · ${v.rules.replaceAll('_',' ')}`;
  $('perspective').textContent='You are '+cap(v.seat);
  if(v.kind==='pass') {
    $('prompt').textContent=v.direction==='keeper'?'Which three cards would you discard?':`What would you pass to ${cap(v.receiver)}?`;
    $('instruction').textContent=v.direction==='keeper'?'Choose three cards for the shared pool. You may receive your own discards back.':`Choose three cards to pass ${v.direction}.`;
  } else if(v.kind==='charge') {
    $('prompt').textContent='What would you charge?';
    const stage=v.phase==='ChargeKeeper1'?'Keeper, before the redraw. Any charge prevents the redraw. ':v.phase==='ChargeKeeper2'?'Keeper, after the redraw. ':'';
    $('instruction').textContent=stage+'Choose any available charge cards, or explicitly choose “No charge.”';
  } else {
    $('prompt').textContent=v.trick.length?'Which card would you play?':'Which card would you lead?';
    $('instruction').textContent='Choose the best card from what you know at this moment.';
  }
  const blind=v.rules.includes('blind'), chain=v.rules.includes('chain'), free=['classic','blind'].includes(v.rules);
  $('rule-detail').textContent=`${cap(v.rules.replaceAll('_',' '))}: ${free?'players may charge without a fixed seat order':'charging follows the fixed seat order'}. ${blind?'Other players’ charge identities stay hidden until play begins.':'Charge cards are public.'} ${chain?'A player may add further charges when charging continues.':'A player finishes its current charging turn after charging; another charge can reopen it.'} Charging ends when all players finish the cycle.`;
  renderPlayers(v);renderTrick(v);renderRemaining(v);renderHistory(v);renderHand();
  cleanDraft=draft();
  if(a)message(`Editing a ${a.status} judgment. Saving creates a new revision.`);
}
async function load(caseId) {
  const version=++requestNumber;busy=true;renderHand();
  try {
    if(!deployed)localStorage.setItem('hearts-judgment-reviewer',reviewer());
    const params=new URLSearchParams({reviewer:reviewer(),kind:$('kind').value,direction:$('direction').value});if(caseId)params.set('case',caseId);
    const query={kind:$('kind').value,direction:$('direction').value};if(caseId)query.case=caseId;
    const data=await (deployed?api('/api/state',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(query)}):api('/api/state?'+params));if(version!==requestNumber)return;
    const totals={pending:0,answered:0,skipped:0};for(const p of Object.values(data.progress))for(const k of Object.keys(totals))totals[k]+=p[k];
    $('progress').textContent=`${totals.answered} answered · ${totals.pending} remaining · ${totals.skipped} skipped`;
    $('recent').replaceChildren();
    for(const r of data.recent){const e=node('button',`${cap(r.kind)} · ${cap(r.direction)} · ${r.status}`,r.case_id===data.case?.id?'current':'');e.append(node('small',new Date(r.created_at).toLocaleString()));e.onclick=()=>navigate(r.case_id);$('recent').append(e);}
    if(!data.recent.length)$('recent').append(node('p','Your saved judgments will appear here.','muted small'));
    $('workspace').hidden=!data.case;$('empty').hidden=Boolean(data.case);
    busy=false;if(data.case)showQuestion(data.case);else current=null;
  } catch(e){message(e.message,true);busy=false;renderHand();}
}
async function save(status) {
  if(!current||busy)return;
  if(status==='answered' && (mode!=='preferred'||!decided||!valid(primary)))return;
  busy=true;renderHand();
  const body={case_id:current.id,...(deployed?{}:{reviewer:reviewer()}),expected_revision:current.revision,status,
    action:primary,alternatives,confidence:$('confidence').value,
    intents:[...$('intents').querySelectorAll('input:checked')].map(e=>e.value),
    suspected_runner:$('runner').value,note:$('note').value,recognized:$('recognized').checked,
    viewing_seconds:Math.min(86400,(performance.now()-started)/1000)};
  try{const result=await api('/api/review',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});current.revision=result.revision;cleanDraft=draft();message(status==='answered'?'Judgment saved.':'Question skipped.');await load();}
  catch(e){busy=false;message(e.message,true);renderHand();}
}
if(deployed){$('reviewer-label').hidden=true;$('lobby-link').hidden=false;$('storage-note').textContent='Saved to your account on this server.';$('participation').textContent='Help improve the bot by choosing what you would do. Your choices and optional explanations will be saved with your account for the site owner to analyze. There is no known correct answer; skip any question you prefer.';}
else $('reviewer').value=localStorage.getItem('hearts-judgment-reviewer')||'local';
for(const [value,label] of Object.entries(names)){const e=node('label',undefined,'checkbox'),input=node('input');input.type='checkbox';input.value=value;e.append(input,node('span',label));$('intents').append(e);}
for(const id of ['reviewer','kind','direction'])$(id).addEventListener('change',()=>navigate());
$('next').onclick=()=>navigate();$('save').onclick=()=>save('answered');$('skip').onclick=()=>save('skipped');
$('decline').onclick=()=>{if(mode==='preferred'){primary=[];decided=true;}else{working=[];altDecided=true;}renderHand();};
$('add-alternative').onclick=()=>{mode='alternative';working=[];altDecided=false;renderHand();};
$('cancel-alternative').onclick=()=>{mode='preferred';renderHand();};
$('keep-alternative').onclick=()=>{if(same(primary,working)||alternatives.some(a=>same(a,working))){message('This choice is already recorded.',true);return;}alternatives.push([...working]);mode='preferred';message('');renderHand();};
document.addEventListener('keydown',e=>{if((e.ctrlKey||e.metaKey)&&e.key==='Enter'){e.preventDefault();if(!$('save').disabled)save('answered');}});
load();

window.addEventListener('beforeunload',e=>{if(dirty()){e.preventDefault();e.returnValue='';}});

// Keep a stalled Builder ID email form open for the user's Continue click.
const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
function createEmailRetry({delayMs=8000,notify=console.log,mode=process.env.KIRO_EMAIL_CONTINUE_MODE||'wait'}={}){
  let retries=0,lastSubmitted=0,waitingAt=0,paused=0,notified=false;
  const finishWait=()=>{if(waitingAt){paused+=Date.now()-waitingAt;waitingAt=0;if(notified)notify('AWS_EMAIL_CONTINUED: page advanced; automation resumed');notified=false;}};
  return {
    submitted(){lastSubmitted=Date.now();if(mode!=='retry')waitingAt=lastSubmitted;notify('AWS_EMAIL_SUBMITTED: initial Continue clicked');},
    pausedMs(){return paused+(waitingAt?Date.now()-waitingAt:0);},
    async handle(page,text,email){
      const url=new URL(page.url());
      if(url.protocol!=='https:'||!['profile.aws.amazon.com','us-east-1.credentials.signin.aws','us-east-1.sso.signin.aws','us-east-1.signin.aws'].includes(url.hostname))return false;
      const captcha=/Security check|Security Verification|Invalid captcha/i.test(text)&&!/successfully solved|Success!/i.test(text);
      // The DOM can advance while the caller still holds the previous ERR-837
      // text snapshot. Re-read on the next iteration instead of treating a
      // disappearing email field as a terminal authentication error.
      if(mode!=='retry'&&lastSubmitted&&!captcha&&(/Get started/i.test(text)||(waitingAt&&/\bERR-837\b/i.test(text)))){
        waitingAt ||= Date.now();
        if(!notified&&(/\bERR-837\b/i.test(text)||Date.now()-lastSubmitted>=delayMs)){
          notified=true;notify('AWS_EMAIL_WAITING: email Continue did not advance; keeping this page open for the user');await page.bringToFront();
        }
        await pause(500);return true;
      }
      if(!/Get started/i.test(text)||captcha){finishWait();return false;}
      const field=page.getByRole('textbox',{name:/^Email$/i});
      const button=page.getByRole('button',{name:/^Continue$|^继续$/});
      if(!await field.isVisible()||await page.locator('input[type=password]').first().isVisible()){finishWait();return false;}
      if(!lastSubmitted)return false;
      if(!/\bERR-837\b/i.test(text))return false;
      const remaining=delayMs-(Date.now()-lastSubmitted);
      if(remaining>0){await pause(Math.min(remaining,500));return true;}
      if(retries>=2)return false;
      if(!await button.isEnabled()){await pause(500);return true;}
      await field.fill(email);
      await button.click();
      lastSubmitted=Date.now();retries++;
      notify(`AWS_EMAIL_RETRY: ${retries}/2; Continue clicked after ERR-837`);
      return true;
    },
  };
}
module.exports={createEmailRetry};

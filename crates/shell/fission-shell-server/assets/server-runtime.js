(function(){
  function setText(selector,text){var el=document.querySelector(selector);if(el)el.textContent=text;}
  function instantiateArtifact(artifact, exportName, onLoaded){
    if(!artifact)return Promise.resolve(false);
    return fetch(artifact).then(function(response){
      if(!response.ok)throw new Error(artifact+' returned '+response.status);
      return response.arrayBuffer();
    }).then(function(bytes){
      return WebAssembly.instantiate(bytes,{});
    }).then(function(result){
      var fn=result.instance&&result.instance.exports&&result.instance.exports[exportName];
      if(typeof fn==='function')fn();
      onLoaded();
      return true;
    });
  }
  document.addEventListener('DOMContentLoaded',function(){
    var manifestEl=document.getElementById('fission-route-manifest');
    if(!manifestEl)return;
    var manifest;
    try{manifest=JSON.parse(manifestEl.textContent||'{}');}catch(error){return;}
    (manifest.workers||[]).forEach(function(worker){
      instantiateArtifact(worker.artifact,'fission_worker_entry',function(){
        setText('[data-fission-semantics="worker-status:'+worker.id+'"]','Worker loaded');
      }).catch(function(error){
        setText('[data-fission-semantics="worker-status:'+worker.id+'"]','Worker failed: '+error.message);
      });
    });
    (manifest.islands||[]).forEach(function(island){
      instantiateArtifact(island.artifact,'fission_island_entry',function(){
        var mount=document.getElementById(island.mount_id)||document.querySelector('[data-fission-semantics="island-status:'+island.id+'"]');
        if(mount)mount.setAttribute('data-fission-island-loaded','true');
        setText('[data-fission-semantics="island-status:'+island.id+'"]','Island loaded');
      }).catch(function(error){
        setText('[data-fission-semantics="island-status:'+island.id+'"]','Island failed: '+error.message);
      });
    });
  });
}());

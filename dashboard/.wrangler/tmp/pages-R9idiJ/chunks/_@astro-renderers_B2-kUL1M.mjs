globalThis.process ??= {}; globalThis.process.env ??= {};
var n,l$1,u$1,i$1,r$1,o$1,e,f$1,c$1,s$1,a$1,p$1={},v$1=[],y$1=/acit|ex(?:s|g|n|p|$)|rph|grid|ows|mnc|ntw|ine[ch]|zoo|^ord|itera/i,d$1=Array.isArray;function w$1(n,l){for(var u in l)n[u]=l[u];return n}function g$1(n){n&&n.parentNode&&n.parentNode.removeChild(n);}function _$1(l,u,t){var i,r,o,e={};for(o in u)"key"==o?i=u[o]:"ref"==o?r=u[o]:e[o]=u[o];if(arguments.length>2&&(e.children=arguments.length>3?n.call(arguments,2):t),"function"==typeof l&&null!=l.defaultProps)for(o in l.defaultProps) void 0===e[o]&&(e[o]=l.defaultProps[o]);return m$1(l,e,i,r,null)}function m$1(n,t,i,r,o){var e={type:n,props:t,key:i,ref:r,__k:null,__:null,__b:0,__e:null,__c:null,constructor:void 0,__v:null==o?++u$1:o,__i:-1,__u:0};return null==o&&null!=l$1.vnode&&l$1.vnode(e),e}function k$1(n){return n.children}function x$1(n,l){this.props=n,this.context=l;}function S$1(n,l){if(null==l)return n.__?S$1(n.__,n.__i+1):null;for(var u;l<n.__k.length;l++)if(null!=(u=n.__k[l])&&null!=u.__e)return u.__e;return "function"==typeof n.type?S$1(n):null}function C$1(n){if(n.__P&&n.__d){var u=n.__v,t=u.__e,i=[],r=[],o=w$1({},u);o.__v=u.__v+1,l$1.vnode&&l$1.vnode(o),z$1(n.__P,o,u,n.__n,n.__P.namespaceURI,32&u.__u?[t]:null,i,null==t?S$1(u):t,!!(32&u.__u),r),o.__v=u.__v,o.__.__k[o.__i]=o,V(i,o,r),u.__e=u.__=null,o.__e!=t&&M$1(o);}}function M$1(n){if(null!=(n=n.__)&&null!=n.__c)return n.__e=n.__c.base=null,n.__k.some(function(l){if(null!=l&&null!=l.__e)return n.__e=n.__c.base=l.__e}),M$1(n)}function $$1(n){(!n.__d&&(n.__d=true)&&i$1.push(n)&&!I$1.__r++||r$1!=l$1.debounceRendering)&&((r$1=l$1.debounceRendering)||o$1)(I$1);}function I$1(){for(var n,l=1;i$1.length;)i$1.length>l&&i$1.sort(e),n=i$1.shift(),l=i$1.length,C$1(n);I$1.__r=0;}function P$1(n,l,u,t,i,r,o,e,f,c,s){var a,h,y,d,w,g,_,m=t&&t.__k||v$1,b=l.length;for(f=A$1(u,l,m,f,b),a=0;a<b;a++)null!=(y=u.__k[a])&&(h=-1!=y.__i&&m[y.__i]||p$1,y.__i=a,g=z$1(n,y,h,i,r,o,e,f,c,s),d=y.__e,y.ref&&h.ref!=y.ref&&(h.ref&&D$1(h.ref,null,y),s.push(y.ref,y.__c||d,y)),null==w&&null!=d&&(w=d),(_=!!(4&y.__u))||h.__k===y.__k?f=H$1(y,f,n,_):"function"==typeof y.type&&void 0!==g?f=g:d&&(f=d.nextSibling),y.__u&=-7);return u.__e=w,f}function A$1(n,l,u,t,i){var r,o,e,f,c,s=u.length,a=s,h=0;for(n.__k=new Array(i),r=0;r<i;r++)null!=(o=l[r])&&"boolean"!=typeof o&&"function"!=typeof o?("string"==typeof o||"number"==typeof o||"bigint"==typeof o||o.constructor==String?o=n.__k[r]=m$1(null,o,null,null,null):d$1(o)?o=n.__k[r]=m$1(k$1,{children:o},null,null,null):void 0===o.constructor&&o.__b>0?o=n.__k[r]=m$1(o.type,o.props,o.key,o.ref?o.ref:null,o.__v):n.__k[r]=o,f=r+h,o.__=n,o.__b=n.__b+1,e=null,-1!=(c=o.__i=T$1(o,u,f,a))&&(a--,(e=u[c])&&(e.__u|=2)),null==e||null==e.__v?(-1==c&&(i>s?h--:i<s&&h++),"function"!=typeof o.type&&(o.__u|=4)):c!=f&&(c==f-1?h--:c==f+1?h++:(c>f?h--:h++,o.__u|=4))):n.__k[r]=null;if(a)for(r=0;r<s;r++)null!=(e=u[r])&&0==(2&e.__u)&&(e.__e==t&&(t=S$1(e)),E$1(e,e));return t}function H$1(n,l,u,t){var i,r;if("function"==typeof n.type){for(i=n.__k,r=0;i&&r<i.length;r++)i[r]&&(i[r].__=n,l=H$1(i[r],l,u,t));return l}n.__e!=l&&(t&&(l&&n.type&&!l.parentNode&&(l=S$1(n)),u.insertBefore(n.__e,l||null)),l=n.__e);do{l=l&&l.nextSibling;}while(null!=l&&8==l.nodeType);return l}function T$1(n,l,u,t){var i,r,o,e=n.key,f=n.type,c=l[u],s=null!=c&&0==(2&c.__u);if(null===c&&null==e||s&&e==c.key&&f==c.type)return u;if(t>(s?1:0))for(i=u-1,r=u+1;i>=0||r<l.length;)if(null!=(c=l[o=i>=0?i--:r++])&&0==(2&c.__u)&&e==c.key&&f==c.type)return o;return  -1}function j$1(n,l,u){"-"==l[0]?n.setProperty(l,null==u?"":u):n[l]=null==u?"":"number"!=typeof u||y$1.test(l)?u:u+"px";}function F$1(n,l,u,t,i){var r,o;n:if("style"==l)if("string"==typeof u)n.style.cssText=u;else {if("string"==typeof t&&(n.style.cssText=t=""),t)for(l in t)u&&l in u||j$1(n.style,l,"");if(u)for(l in u)t&&u[l]==t[l]||j$1(n.style,l,u[l]);}else if("o"==l[0]&&"n"==l[1])r=l!=(l=l.replace(f$1,"$1")),o=l.toLowerCase(),l=o in n||"onFocusOut"==l||"onFocusIn"==l?o.slice(2):l.slice(2),n.l||(n.l={}),n.l[l+r]=u,u?t?u.u=t.u:(u.u=c$1,n.addEventListener(l,r?a$1:s$1,r)):n.removeEventListener(l,r?a$1:s$1,r);else {if("http://www.w3.org/2000/svg"==i)l=l.replace(/xlink(H|:h)/,"h").replace(/sName$/,"s");else if("width"!=l&&"height"!=l&&"href"!=l&&"list"!=l&&"form"!=l&&"tabIndex"!=l&&"download"!=l&&"rowSpan"!=l&&"colSpan"!=l&&"role"!=l&&"popover"!=l&&l in n)try{n[l]=null==u?"":u;break n}catch(n){}"function"==typeof u||(null==u||false===u&&"-"!=l[4]?n.removeAttribute(l):n.setAttribute(l,"popover"==l&&1==u?"":u));}}function O$1(n){return function(u){if(this.l){var t=this.l[u.type+n];if(null==u.t)u.t=c$1++;else if(u.t<t.u)return;return t(l$1.event?l$1.event(u):u)}}}function z$1(n,u,t,i,r,o,e,f,c,s){var a,h,p,y,_,m,b,S,C,M,$,I,A,H,L,T=u.type;if(void 0!==u.constructor)return null;128&t.__u&&(c=!!(32&t.__u),o=[f=u.__e=t.__e]),(a=l$1.__b)&&a(u);n:if("function"==typeof T)try{if(S=u.props,C="prototype"in T&&T.prototype.render,M=(a=T.contextType)&&i[a.__c],$=a?M?M.props.value:a.__:i,t.__c?b=(h=u.__c=t.__c).__=h.__E:(C?u.__c=h=new T(S,$):(u.__c=h=new x$1(S,$),h.constructor=T,h.render=G$1),M&&M.sub(h),h.state||(h.state={}),h.__n=i,p=h.__d=!0,h.__h=[],h._sb=[]),C&&null==h.__s&&(h.__s=h.state),C&&null!=T.getDerivedStateFromProps&&(h.__s==h.state&&(h.__s=w$1({},h.__s)),w$1(h.__s,T.getDerivedStateFromProps(S,h.__s))),y=h.props,_=h.state,h.__v=u,p)C&&null==T.getDerivedStateFromProps&&null!=h.componentWillMount&&h.componentWillMount(),C&&null!=h.componentDidMount&&h.__h.push(h.componentDidMount);else {if(C&&null==T.getDerivedStateFromProps&&S!==y&&null!=h.componentWillReceiveProps&&h.componentWillReceiveProps(S,$),u.__v==t.__v||!h.__e&&null!=h.shouldComponentUpdate&&!1===h.shouldComponentUpdate(S,h.__s,$)){u.__v!=t.__v&&(h.props=S,h.state=h.__s,h.__d=!1),u.__e=t.__e,u.__k=t.__k,u.__k.some(function(n){n&&(n.__=u);}),v$1.push.apply(h.__h,h._sb),h._sb=[],h.__h.length&&e.push(h);break n}null!=h.componentWillUpdate&&h.componentWillUpdate(S,h.__s,$),C&&null!=h.componentDidUpdate&&h.__h.push(function(){h.componentDidUpdate(y,_,m);});}if(h.context=$,h.props=S,h.__P=n,h.__e=!1,I=l$1.__r,A=0,C)h.state=h.__s,h.__d=!1,I&&I(u),a=h.render(h.props,h.state,h.context),v$1.push.apply(h.__h,h._sb),h._sb=[];else do{h.__d=!1,I&&I(u),a=h.render(h.props,h.state,h.context),h.state=h.__s;}while(h.__d&&++A<25);h.state=h.__s,null!=h.getChildContext&&(i=w$1(w$1({},i),h.getChildContext())),C&&!p&&null!=h.getSnapshotBeforeUpdate&&(m=h.getSnapshotBeforeUpdate(y,_)),H=null!=a&&a.type===k$1&&null==a.key?q$1(a.props.children):a,f=P$1(n,d$1(H)?H:[H],u,t,i,r,o,e,f,c,s),h.base=u.__e,u.__u&=-161,h.__h.length&&e.push(h),b&&(h.__E=h.__=null);}catch(n){if(u.__v=null,c||null!=o)if(n.then){for(u.__u|=c?160:128;f&&8==f.nodeType&&f.nextSibling;)f=f.nextSibling;o[o.indexOf(f)]=null,u.__e=f;}else {for(L=o.length;L--;)g$1(o[L]);N$1(u);}else u.__e=t.__e,u.__k=t.__k,n.then||N$1(u);l$1.__e(n,u,t);}else null==o&&u.__v==t.__v?(u.__k=t.__k,u.__e=t.__e):f=u.__e=B(t.__e,u,t,i,r,o,e,c,s);return (a=l$1.diffed)&&a(u),128&u.__u?void 0:f}function N$1(n){n&&(n.__c&&(n.__c.__e=true),n.__k&&n.__k.some(N$1));}function V(n,u,t){for(var i=0;i<t.length;i++)D$1(t[i],t[++i],t[++i]);l$1.__c&&l$1.__c(u,n),n.some(function(u){try{n=u.__h,u.__h=[],n.some(function(n){n.call(u);});}catch(n){l$1.__e(n,u.__v);}});}function q$1(n){return "object"!=typeof n||null==n||n.__b>0?n:d$1(n)?n.map(q$1):w$1({},n)}function B(u,t,i,r,o,e,f,c,s){var a,h,v,y,w,_,m,b=i.props||p$1,k=t.props,x=t.type;if("svg"==x?o="http://www.w3.org/2000/svg":"math"==x?o="http://www.w3.org/1998/Math/MathML":o||(o="http://www.w3.org/1999/xhtml"),null!=e)for(a=0;a<e.length;a++)if((w=e[a])&&"setAttribute"in w==!!x&&(x?w.localName==x:3==w.nodeType)){u=w,e[a]=null;break}if(null==u){if(null==x)return document.createTextNode(k);u=document.createElementNS(o,x,k.is&&k),c&&(l$1.__m&&l$1.__m(t,e),c=false),e=null;}if(null==x)b===k||c&&u.data==k||(u.data=k);else {if(e=e&&n.call(u.childNodes),!c&&null!=e)for(b={},a=0;a<u.attributes.length;a++)b[(w=u.attributes[a]).name]=w.value;for(a in b)w=b[a],"dangerouslySetInnerHTML"==a?v=w:"children"==a||a in k||"value"==a&&"defaultValue"in k||"checked"==a&&"defaultChecked"in k||F$1(u,a,null,w,o);for(a in k)w=k[a],"children"==a?y=w:"dangerouslySetInnerHTML"==a?h=w:"value"==a?_=w:"checked"==a?m=w:c&&"function"!=typeof w||b[a]===w||F$1(u,a,w,b[a],o);if(h)c||v&&(h.__html==v.__html||h.__html==u.innerHTML)||(u.innerHTML=h.__html),t.__k=[];else if(v&&(u.innerHTML=""),P$1("template"==t.type?u.content:u,d$1(y)?y:[y],t,i,r,"foreignObject"==x?"http://www.w3.org/1999/xhtml":o,e,f,e?e[0]:i.__k&&S$1(i,0),c,s),null!=e)for(a=e.length;a--;)g$1(e[a]);c||(a="value","progress"==x&&null==_?u.removeAttribute("value"):null!=_&&(_!==u[a]||"progress"==x&&!_||"option"==x&&_!=b[a])&&F$1(u,a,_,b[a],o),a="checked",null!=m&&m!=u[a]&&F$1(u,a,m,b[a],o));}return u}function D$1(n,u,t){try{if("function"==typeof n){var i="function"==typeof n.__u;i&&n.__u(),i&&null==u||(n.__u=n(u));}else n.current=u;}catch(n){l$1.__e(n,t);}}function E$1(n,u,t){var i,r;if(l$1.unmount&&l$1.unmount(n),(i=n.ref)&&(i.current&&i.current!=n.__e||D$1(i,null,u)),null!=(i=n.__c)){if(i.componentWillUnmount)try{i.componentWillUnmount();}catch(n){l$1.__e(n,u);}i.base=i.__P=null;}if(i=n.__k)for(r=0;r<i.length;r++)i[r]&&E$1(i[r],u,t||"function"!=typeof n.type);t||g$1(n.__e),n.__c=n.__=n.__e=void 0;}function G$1(n,l,u){return this.constructor(n,u)}n=v$1.slice,l$1={__e:function(n,l,u,t){for(var i,r,o;l=l.__;)if((i=l.__c)&&!i.__)try{if((r=i.constructor)&&null!=r.getDerivedStateFromError&&(i.setState(r.getDerivedStateFromError(n)),o=i.__d),null!=i.componentDidCatch&&(i.componentDidCatch(n,t||{}),o=i.__d),o)return i.__E=i}catch(l){n=l;}throw n}},u$1=0,x$1.prototype.setState=function(n,l){var u;u=null!=this.__s&&this.__s!=this.state?this.__s:this.__s=w$1({},this.state),"function"==typeof n&&(n=n(w$1({},u),this.props)),n&&w$1(u,n),null!=n&&this.__v&&(l&&this._sb.push(l),$$1(this));},x$1.prototype.forceUpdate=function(n){this.__v&&(this.__e=true,n&&this.__h.push(n),$$1(this));},x$1.prototype.render=k$1,i$1=[],o$1="function"==typeof Promise?Promise.prototype.then.bind(Promise.resolve()):setTimeout,e=function(n,l){return n.__v.__b-l.__v.__b},I$1.__r=0,f$1=/(PointerCapture)$|Capture$/i,c$1=0,s$1=O$1(false),a$1=O$1(true);

var r="diffed",o="__c",i="__s",a="__c",c="__k",u="__d",s="__s",l=/[\s\n\\/='"\0<>]/,f=/^(xlink|xmlns|xml)([A-Z])/,p=/^(?:accessK|auto[A-Z]|cell|ch|col|cont|cross|dateT|encT|form[A-Z]|frame|hrefL|inputM|maxL|minL|noV|playsI|popoverT|readO|rowS|src[A-Z]|tabI|useM|item[A-Z])/,h=/^ac|^ali|arabic|basel|cap|clipPath$|clipRule$|color|dominant|enable|fill|flood|font|glyph[^R]|horiz|image|letter|lighting|marker[^WUH]|overline|panose|pointe|paint|rendering|shape|stop|strikethrough|stroke|text[^L]|transform|underline|unicode|units|^v[^i]|^w|^xH/,d=new Set(["draggable","spellcheck"]);function v(e){ void 0!==e.__g?e.__g|=8:e[u]=true;}function m(e){ void 0!==e.__g?e.__g&=-9:e[u]=false;}function y(e){return void 0!==e.__g?!!(8&e.__g):true===e[u]}var _=/["&<]/;function g(e){if(0===e.length||false===_.test(e))return e;for(var t=0,n=0,r="",o="";n<e.length;n++){switch(e.charCodeAt(n)){case 34:o="&quot;";break;case 38:o="&amp;";break;case 60:o="&lt;";break;default:continue}n!==t&&(r+=e.slice(t,n)),r+=o,t=n+1;}return n!==t&&(r+=e.slice(t,n)),r}var b={},x=new Set(["animation-iteration-count","border-image-outset","border-image-slice","border-image-width","box-flex","box-flex-group","box-ordinal-group","column-count","fill-opacity","flex","flex-grow","flex-negative","flex-order","flex-positive","flex-shrink","flood-opacity","font-weight","grid-column","grid-row","line-clamp","line-height","opacity","order","orphans","stop-opacity","stroke-dasharray","stroke-dashoffset","stroke-miterlimit","stroke-opacity","stroke-width","tab-size","widows","z-index","zoom"]),k=/[A-Z]/g;function w(e){var t="";for(var n in e){var r=e[n];if(null!=r&&""!==r){var o="-"==n[0]?n:b[n]||(b[n]=n.replace(k,"-$&").toLowerCase()),i=";";"number"!=typeof r||o.startsWith("--")||x.has(o)||(i="px;"),t=t+o+":"+r+i;}}return t||void 0}function C(){this.__d=true;}function A(e,t){return {__v:e,context:t,props:e.props,setState:C,forceUpdate:C,__d:true,__h:new Array(0)}}function S(e,t,n){if(!e.s){if(n instanceof L){if(!n.s)return void(n.o=S.bind(null,e,t));1&t&&(t=n.s),n=n.v;}if(n&&n.then)return void n.then(S.bind(null,e,t),S.bind(null,e,2));e.s=t,e.v=n;const r=e.o;r&&r(e);}}var L=/*#__PURE__*/function(){function e(){}return e.prototype.then=function(t,n){var r=new e,o=this.s;if(o){var i=1&o?t:n;if(i){try{S(r,1,i(this.v));}catch(e){S(r,2,e);}return r}return this}return this.o=function(e){try{var o=e.v;1&e.s?S(r,1,t?t(o):o):n?S(r,1,n(o)):S(r,2,o);}catch(e){S(r,2,e);}},r},e}();function E(e){return e instanceof L&&1&e.s}function j(e,t,n){for(var r;;){var o=e();if(E(o)&&(o=o.v),!o)return i;if(o.then){r=0;break}var i=n();if(i&&i.then){if(!E(i)){r=1;break}i=i.s;}var a; }var c=new L,u=S.bind(null,c,2);return (0===r?o.then(l):1===r?i.then(s):a.then(f)).then(void 0,u),c;function s(r){i=r;do{if(!(o=e())||E(o)&&!o.v)return void S(c,1,i);if(o.then)return void o.then(l).then(void 0,u);E(i=n())&&(i=i.v);}while(!i||!i.then);i.then(s).then(void 0,u);}function l(e){e?(i=n())&&i.then?i.then(s).then(void 0,u):s(i):S(c,1,i);}function f(){(o=e())?o.then?o.then(l).then(void 0,u):l(o):S(c,1,i);}}function T(e,t){try{var n=e();}catch(e){return t(true,e)}return n&&n.then?n.then(t.bind(null,false),t.bind(null,true)):t(false,n)}var D,P,$,U,Z=function(a,u){try{var s=l$1[i];l$1[i]=!0,D=l$1.__b,P=l$1[r],$=l$1.__r,U=l$1.unmount;var l=_$1(k$1,null);return l[c]=[a],Promise.resolve(T(function(){return Promise.resolve(O(a,u||F,!1,void 0,l,!0,void 0)).then(function(e){var t,n=function(){if(W(e)){var n=function(){var e=o.join(H);return t=1,e},r=0,o=e,i=j(function(){return !!o.some(function(e){return e&&"function"==typeof e.then})&&r++<25},void 0,function(){return Promise.resolve(Promise.all(o)).then(function(e){o=e.flat();})});return i&&i.then?i.then(n):n()}}();return n&&n.then?n.then(function(n){return t?n:e}):t?n:e})},function(t,n){if(l$1[o]&&l$1[o](a,M),l$1[i]=s,M.length=0,t)throw n;return n}))}catch(e){return Promise.reject(e)}},F={},M=[],W=Array.isArray,z=Object.assign,H="",N="\x3c!--$s--\x3e",q="\x3c!--/$s--\x3e";function I(e,t){var n,r=e.type,o=true;return e[a]?(o=false,(n=e[a]).state=n[s]):n=new r(e.props,t),e[a]=n,n.__v=e,n.props=e.props,n.context=t,v(n),null==n.state&&(n.state=F),null==n[s]&&(n[s]=n.state),r.getDerivedStateFromProps?n.state=z({},n.state,r.getDerivedStateFromProps(n.props,n.state)):o&&n.componentWillMount?(n.componentWillMount(),n.state=n[s]!==n.state?n[s]:n.state):!o&&n.componentWillUpdate&&n.componentWillUpdate(),$&&$(e),n.render(n.props,n.state,t)}function O(t,r,o,i,u,_,b){if(null==t||true===t||false===t||t===H)return H;var x=typeof t;if("object"!=x)return "function"==x?H:"string"==x?g(t):t+H;if(W(t)){var k,C=H;u[c]=t;for(var S=t.length,L=0;L<S;L++){var E=t[L];if(null!=E&&"boolean"!=typeof E){var j,T=O(E,r,o,i,u,_,b);"string"==typeof T?C+=T:(k||(k=new Array(S)),C&&k.push(C),C=H,W(T)?(j=k).push.apply(j,T):k.push(T));}}return k?(C&&k.push(C),k):C}if(void 0!==t.constructor)return H;t.__=u,D&&D(t);var Z=t.type,M=t.props;if("function"==typeof Z){var B,V,K,J=r;if(Z===k$1){if("tpl"in M){for(var Q=H,X=0;X<M.tpl.length;X++)if(Q+=M.tpl[X],M.exprs&&X<M.exprs.length){var Y=M.exprs[X];if(null==Y)continue;"object"!=typeof Y||void 0!==Y.constructor&&!W(Y)?Q+=Y:Q+=O(Y,r,o,i,t,_,b);}return Q}if("UNSTABLE_comment"in M)return "\x3c!--"+g(M.UNSTABLE_comment)+"--\x3e";V=M.children;}else {if(null!=(B=Z.contextType)){var ee=r[B.__c];J=ee?ee.props.value:B.__;}var te=Z.prototype&&"function"==typeof Z.prototype.render;if(te)V=/**#__NOINLINE__**/I(t,J),K=t[a];else {t[a]=K=/**#__NOINLINE__**/A(t,J);for(var ne=0;y(K)&&ne++<25;){m(K),$&&$(t);try{V=Z.call(K,M,J);}catch(e){throw e&&"function"==typeof e.then&&(t._suspended=true),e}}v(K);}if(null!=K.getChildContext&&(r=z({},r,K.getChildContext())),te&&l$1.errorBoundaries&&(Z.getDerivedStateFromError||K.componentDidCatch)){V=null!=V&&V.type===k$1&&null==V.key&&null==V.props.tpl?V.props.children:V;try{return O(V,r,o,i,t,_,!1)}catch(e){return Z.getDerivedStateFromError&&(K[s]=Z.getDerivedStateFromError(e)),K.componentDidCatch&&K.componentDidCatch(e,F),y(K)?(V=I(t,r),null!=(K=t[a]).getChildContext&&(r=z({},r,K.getChildContext())),O(V=null!=V&&V.type===k$1&&null==V.key&&null==V.props.tpl?V.props.children:V,r,o,i,t,_,b)):H}finally{P&&P(t),U&&U(t);}}}V=null!=V&&V.type===k$1&&null==V.key&&null==V.props.tpl?V.props.children:V;try{var re=O(V,r,o,i,t,_,b);return P&&P(t),l$1.unmount&&l$1.unmount(t),t._suspended?"string"==typeof re?N+re+q:W(re)?(re.unshift(N),re.push(q),re):re.then(function(e){return N+e+q}):re}catch(n){if(!n||"function"!=typeof n.then)throw n;return n.then(function e(){try{var n=O(V,r,o,i,t,_,b);return t._suspended?N+n+q:n}catch(t){if(!t||"function"!=typeof t.then)throw t;return t.then(e)}})}}var ae,ce="<"+Z,ue=H;for(var se in M){var le=M[se];if("function"!=typeof(le=G(le)?le.value:le)||"class"===se||"className"===se){switch(se){case "children":ae=le;continue;case "key":case "ref":case "__self":case "__source":continue;case "htmlFor":if("for"in M)continue;se="for";break;case "className":if("class"in M)continue;se="class";break;case "defaultChecked":se="checked";break;case "defaultSelected":se="selected";break;case "defaultValue":case "value":switch(se="value",Z){case "textarea":ae=le;continue;case "select":i=le;continue;case "option":i!=le||"selected"in M||(ce+=" selected");}break;case "dangerouslySetInnerHTML":ue=le&&le.__html;continue;case "style":"object"==typeof le&&(le=w(le));break;case "acceptCharset":se="accept-charset";break;case "httpEquiv":se="http-equiv";break;default:if(f.test(se))se=se.replace(f,"$1:$2").toLowerCase();else {if(l.test(se))continue;"-"!==se[4]&&!d.has(se)||null==le?o?h.test(se)&&(se="panose1"===se?"panose-1":se.replace(/([A-Z])/g,"-$1").toLowerCase()):p.test(se)&&(se=se.toLowerCase()):le+=H;}}null!=le&&false!==le&&(ce=true===le||le===H?ce+" "+se:ce+" "+se+'="'+("string"==typeof le?g(le):le+H)+'"');}}if(l.test(Z))throw new Error(Z+" is not a valid HTML tag name in "+ce+">");if(ue||("string"==typeof ae?ue=g(ae):null!=ae&&false!==ae&&true!==ae&&(ue=O(ae,r,"svg"===Z||"foreignObject"!==Z&&o,i,t,_,b))),P&&P(t),U&&U(t),!ue&&R.has(Z))return ce+"/>";var fe="</"+Z+">",pe=ce+">";return W(ue)?[pe].concat(ue,[fe]):"string"!=typeof ue?[pe,ue,fe]:pe+ue+fe}var R=new Set(["area","base","br","col","command","embed","hr","img","input","keygen","link","meta","param","source","track","wbr"]);function G(e){return null!==e&&"object"==typeof e&&"function"==typeof e.peek&&"value"in e}

const contexts = /* @__PURE__ */ new WeakMap();
function getContext(result) {
  if (contexts.has(result)) {
    return contexts.get(result);
  }
  let ctx = {
    c: 0,
    get id() {
      return "p" + this.c.toString();
    },
    signals: /* @__PURE__ */ new Map(),
    propsToSignals: /* @__PURE__ */ new Map()
  };
  contexts.set(result, ctx);
  return ctx;
}
function incrementId(ctx) {
  let id = ctx.id;
  ctx.c++;
  return id;
}

function isSignal(x) {
  return x != null && typeof x === "object" && typeof x.peek === "function" && "value" in x;
}
function restoreSignalsOnProps(ctx, props) {
  let propMap;
  if (ctx.propsToSignals.has(props)) {
    propMap = ctx.propsToSignals.get(props);
  } else {
    propMap = /* @__PURE__ */ new Map();
    ctx.propsToSignals.set(props, propMap);
  }
  for (const [key, signal] of propMap) {
    props[key] = signal;
  }
  return propMap;
}
function serializeSignals(ctx, props, attrs, map) {
  const signals = {};
  for (const [key, value] of Object.entries(props)) {
    const isPropArray = Array.isArray(value);
    const isPropObject = !isSignal(value) && typeof props[key] === "object" && props[key] !== null && !isPropArray;
    if (isPropObject || isPropArray) {
      const values = isPropObject ? Object.keys(props[key]) : value;
      values.forEach((valueKey, valueIndex) => {
        const signal = isPropObject ? props[key][valueKey] : valueKey;
        if (isSignal(signal)) {
          const keyOrIndex = isPropObject ? valueKey.toString() : valueIndex;
          props[key] = isPropObject ? Object.assign({}, props[key], { [keyOrIndex]: signal.peek() }) : props[key].map(
            (v, i) => i === valueIndex ? [signal.peek(), i] : v
          );
          const currentMap = map.get(key) || [];
          map.set(key, [...currentMap, [signal, keyOrIndex]]);
          const currentSignals = signals[key] || [];
          signals[key] = [...currentSignals, [getSignalId(ctx, signal), keyOrIndex]];
        }
      });
    } else if (isSignal(value)) {
      props[key] = value.peek();
      map.set(key, value);
      signals[key] = getSignalId(ctx, value);
    }
  }
  if (Object.keys(signals).length) {
    attrs["data-preact-signals"] = JSON.stringify(signals);
  }
}
function getSignalId(ctx, item) {
  let id = ctx.signals.get(item);
  if (!id) {
    id = incrementId(ctx);
    ctx.signals.set(item, id);
  }
  return id;
}

const StaticHtml = ({ value, name, hydrate = true }) => {
  if (!value) return null;
  const tagName = hydrate ? "astro-slot" : "astro-static-slot";
  return _$1(tagName, { name, dangerouslySetInnerHTML: { __html: value } });
};
StaticHtml.shouldComponentUpdate = () => false;
var static_html_default = StaticHtml;

const slotName = (str) => str.trim().replace(/[-_]([a-z])/g, (_, w) => w.toUpperCase());
let originalConsoleError;
let consoleFilterRefs = 0;
async function check(Component, props, children) {
  if (typeof Component !== "function") return false;
  if (Component.name === "QwikComponent") return false;
  if (Component.prototype != null && typeof Component.prototype.render === "function") {
    return x$1.isPrototypeOf(Component);
  }
  useConsoleFilter();
  try {
    const { html } = await renderToStaticMarkup.call(this, Component, props, children, void 0);
    if (typeof html !== "string") {
      return false;
    }
    return html == "" ? false : !html.includes("<undefined>");
  } catch {
    return false;
  } finally {
    finishUsingConsoleFilter();
  }
}
function shouldHydrate(metadata) {
  return metadata?.astroStaticSlot ? !!metadata.hydrate : true;
}
async function renderToStaticMarkup(Component, props, { default: children, ...slotted }, metadata) {
  const ctx = getContext(this.result);
  const slots = {};
  for (const [key, value] of Object.entries(slotted)) {
    const name = slotName(key);
    slots[name] = _$1(static_html_default, {
      hydrate: shouldHydrate(metadata),
      value,
      name
    });
  }
  let propsMap = restoreSignalsOnProps(ctx, props);
  const newProps = { ...props, ...slots };
  const attrs = {};
  serializeSignals(ctx, props, attrs, propsMap);
  const vNode = _$1(
    Component,
    newProps,
    children != null ? _$1(static_html_default, {
      hydrate: shouldHydrate(metadata),
      value: children
    }) : children
  );
  const html = await Z(vNode);
  return { attrs, html };
}
function useConsoleFilter() {
  consoleFilterRefs++;
  if (!originalConsoleError) {
    originalConsoleError = console.error;
    try {
      console.error = filteredConsoleError;
    } catch {
    }
  }
}
function finishUsingConsoleFilter() {
  consoleFilterRefs--;
}
function filteredConsoleError(msg, ...rest) {
  if (consoleFilterRefs > 0 && typeof msg === "string") {
    const isKnownReactHookError = msg.includes("Warning: Invalid hook call.") && msg.includes("https://reactjs.org/link/invalid-hook-call");
    if (isKnownReactHookError) return;
  }
  originalConsoleError(msg, ...rest);
}
const renderer = {
  name: "@astrojs/preact",
  check,
  renderToStaticMarkup,
  supportsAstroStaticSlot: true
};
var server_default = renderer;

const renderers = [Object.assign({"name":"@astrojs/preact","clientEntrypoint":"@astrojs/preact/client.js","serverEntrypoint":"@astrojs/preact/server.js"}, { ssr: server_default }),];

export { l$1 as l, renderers as r };

(this.webpackJsonpterritories=this.webpackJsonpterritories||[]).push([[1],{329:function(e,t,n){"use strict";var a=n(354),r=n(32),o=n(35),l=n(34),i=n(33),c=n(0),s=n.n(c),u=n(94),h=n.n(u),d=n(314),p=n(16),f=n(12);function m(){var e=Object(p.a)(["\n  display: table-cell;\n  padding: ","px;\n  border: ","px solid ",";\n"]);return m=function(){return e},e}function v(){var e=Object(p.a)(["\n  display: table-row;\n"]);return v=function(){return e},e}function g(){var e=Object(p.a)(["\n  display: table;\n  border: ","px solid ",";\n"]);return g=function(){return e},e}var y=f.c.div(g(),1,(function(e){return e.theme.colors.gray})),b=f.c.div(v()),w=f.c.div(m(),(function(e){return e.cellRadius?e.cellRadius-1:10}),1,(function(e){return e.theme.colors.gray}));function x(){var e=Object(p.a)(["\n  background-color: ",";\n"]);return x=function(){return e},e}function C(){var e=Object(p.a)(["\n  background-color: ",";\n"]);return C=function(){return e},e}function j(){var e=Object(p.a)(["\n  background-color: ",";\n"]);return j=function(){return e},e}function O(){var e=Object(p.a)(["\n  position: absolute;\n  top: 0;\n  left: 0;\n  height: 100%;\n  width: 100%;\n  z-index: 1;\n  opacity: 0.5;\n"]);return O=function(){return e},e}function M(){var e=Object(p.a)(["\n  position: relative;\n  background-color: ",";\n"]);return M=function(){return e},e}var I=Object(f.c)(w)(M(),(function(e){return"PLAYER_1"===e.type?e.theme.colors.player1:"PLAYER_2"===e.type?e.theme.colors.player2:"inherit"})),R=f.c.div(O()),E=Object(f.c)(R)(j(),(function(e){return e.theme.colors.green})),k=Object(f.c)(R)(C(),(function(e){return e.theme.colors.red})),D=Object(f.c)(R)(x(),(function(e){return e.theme.colors.yellow})),S=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var a=arguments.length,o=new Array(a),l=0;l<a;l++)o[l]=arguments[l];return(e=t.call.apply(t,[this].concat(o))).state={touchX:-1,touchY:-1,touchDraggedColumnIndex:-1,touchDraggedRowIndex:-1},e.handleCellMouseEnter=function(){var t=e.props,n=t.rowIndex,a=t.columnIndex;(0,t.onMouseEnter)({rowIndex:n,columnIndex:a})},e.handleCellClick=function(){var t=e.props,n=t.rowIndex,a=t.columnIndex;(0,t.onClick)({rowIndex:n,columnIndex:a})},e.handleCellTouchStart=function(t){var n=t.touches;1===n.length&&e.setState({touchX:n[0].clientX,touchY:n[0].clientY}),e.handleCellMouseEnter()},e.handleCellTouchMove=function(t){var n=t.touches;if(!(n.length<1)){var a=e.state,r=a.touchX,o=a.touchY;if(-1!==r&&-1!==o){var l=n[0],i=l.clientX,c=l.clientY,s=2*e.props.cellRadius,u=Math.floor((r-i)/s),h=Math.floor((o-c)/s);if(0!==Math.abs(u)||0!==Math.abs(h)){var d=e.props,p=d.columnIndex,f=d.rowIndex,m=d.boardWidth,v=d.boardHeight,g=d.onMouseEnter,y=p-u,b=f-h;y<0||y>=m||b<0||b>=v||(e.setState({touchDraggedColumnIndex:y,touchDraggedRowIndex:b}),g({columnIndex:y,rowIndex:b}))}}}},e.handleCellTouchEnd=function(t){t&&t.preventDefault();var n=e.state,a=n.touchDraggedColumnIndex,r=n.touchDraggedRowIndex,o=e.props.onClick;-1===a&&-1===r||o({rowIndex:r,columnIndex:a})},e}return Object(o.a)(n,[{key:"render",value:function(){var e=this.props,t=e.canDrop,n=e.value,a=e.rowIndex,r=e.columnIndex,o=e.cellRadius,l=e.selectedRowIndex,i=e.selectedColumnIndex,c=e.rectangleWidth,u=e.rectangleHeight,h=e.potentiallyOccupiedCells,p=d.inRectangle({rowIndex:a,columnIndex:r,selectedRowIndex:l,selectedColumnIndex:i,rectangleWidth:c,rectangleHeight:u}),f=!1;return p||(f=h.some((function(e){return d.inRectangle({rowIndex:a,columnIndex:r,selectedRowIndex:e.rowIndex,selectedColumnIndex:e.columnIndex,rectangleWidth:c,rectangleHeight:u})}))),s.a.createElement(I,{cellRadius:o,type:d.isOccupiedByPlayerOneCell(n)?"PLAYER_1":d.isOccupiedByPlayerTwoCell(n)?"PLAYER_2":"EMPTY",onMouseEnter:this.handleCellMouseEnter,onClick:this.handleCellClick,onTouchStart:this.handleCellTouchStart,onTouchMove:this.handleCellTouchMove,onTouchEnd:this.handleCellTouchEnd},p&&(t?s.a.createElement(E,null):s.a.createElement(k,null)),d.isEmptyCell(n)&&f&&s.a.createElement(D,null))}}]),n}(c.Component),T=n(443),P=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var a=arguments.length,o=new Array(a),l=0;l<a;l++)o[l]=arguments[l];return(e=t.call.apply(t,[this].concat(o))).renderRow=function(t){var n=t.rows,a=t.row,r=t.rowIndex,o=t.rowRenderer,l=t.rowClassName,i=t.rowStyle,c=t.cellRenderer,u=t.cellRadius,h=t.cellClassName,d=t.cellStyle;return o?o({rows:n,value:a,rowIndex:r,rowClassName:l,rowStyle:i,cellRenderer:c,cellRadius:u,cellClassName:h,cellStyle:d}):s.a.createElement(b,{key:"rectangle-row-".concat(r),className:l,style:i},a.map((function(t,o){return e.renderCell({rows:n,row:a,rowIndex:r,columnIndex:o,cell:t,cellRenderer:c,cellRadius:u,cellClassName:h,cellStyle:d})})))},e.renderCell=function(e){var t=e.rows,n=e.row,a=e.rowIndex,r=e.columnIndex,o=e.cell,l=e.cellRenderer,i=e.cellRadius,c=e.cellClassName,u=e.cellStyle;return l?l({rows:t,row:n,rowIndex:a,columnIndex:r,value:o,cellRadius:i,cellClassName:c,cellStyle:u}):s.a.createElement(w,{key:"rectangle-cell-".concat(r),className:c,style:u,cellRadius:i})},e}return Object(o.a)(n,[{key:"render",value:function(){var e=this,t=this.props,n=t.rows,a=t.rowClassName,r=t.rowStyle,o=t.cellClassName,l=t.cellStyle,i=t.cellRadius,c=t.rowRenderer,u=t.cellRenderer,h=Object(T.a)(t,["rows","rowClassName","rowStyle","cellClassName","cellStyle","cellRadius","rowRenderer","cellRenderer"]);if(!Array.isArray(n))throw new Error("'rows' is a required argument in format [[ 0, 1 ], [0, 'any']]");return s.a.createElement(y,h,n.map((function(t,s){return e.renderRow({rows:n,row:t,rowIndex:s,rowRenderer:c,rowClassName:a,rowStyle:r,cellRenderer:u,cellRadius:i,cellClassName:o,cellStyle:l})})))}}]),n}(c.Component),A=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var a=arguments.length,o=new Array(a),l=0;l<a;l++)o[l]=arguments[l];return(e=t.call.apply(t,[this].concat(o))).state={selectedRowIndex:-1,selectedColumnIndex:-1,canDrop:!1,potentiallyOccupiedCells:[]},e.handleCellMouseEnter=function(t){var n=t.rowIndex,a=t.columnIndex,r=e.props,o=r.rows,l=r.rectangleHeight,i=r.rectangleWidth,c=r.currentPlayer;if(!r.disabled){var s=d.canDropRectangle({rowIndex:n,columnIndex:a,value:o[n][a],rows:o,rectangleHeight:l,rectangleWidth:i,currentPlayer:c});e.setState({selectedRowIndex:n,selectedColumnIndex:a,canDrop:s})}},e.handleBoardMouseLeave=function(){e.setState({selectedRowIndex:-1,selectedColumnIndex:-1})},e.handleDropRectangle=function(t){var n=t.rowIndex,a=t.columnIndex,r=e.props.disabled,o=e.state.canDrop,l=e.props,i=l.onDropRectangle,c=l.rectangleHeight,s=l.rectangleWidth;!r&&o&&i({rowIndex:n,columnIndex:a,rectangleHeight:c,rectangleWidth:s}),e.handleBoardMouseLeave()},e}return Object(o.a)(n,[{key:"componentDidUpdate",value:function(e){var t=this.props,n=t.currentPlayer,a=t.rectangleHeight,r=t.rectangleWidth,o=t.rows;if(e.rectangleHeight!==a||e.rectangleWidth!==r||e.currentPlayer!==n){var l=d.findPotentiallyOccupiedRectangles({currentPlayer:n,rectangleHeight:a,rectangleWidth:r,rows:o});this.setState({potentiallyOccupiedCells:l})}}},{key:"render",value:function(){var e=this,t=this.props,n=t.rows,a=t.cellRadius,r=t.rectangleWidth,o=t.rectangleHeight,l=this.state,i=l.selectedRowIndex,c=l.selectedColumnIndex,u=l.canDrop,h=l.potentiallyOccupiedCells;return s.a.createElement(P,{rows:n,cellRenderer:function(t){var l=t.value,d=t.rowIndex,p=t.columnIndex;return s.a.createElement(S,{key:"board-".concat(d,"-").concat(p),canDrop:u,value:l,rowIndex:d,columnIndex:p,selectedRowIndex:i,selectedColumnIndex:c,rectangleHeight:o,rectangleWidth:r,cellRadius:a,potentiallyOccupiedCells:h,boardHeight:n.length,boardWidth:n.length>0?n[0].length:0,onMouseEnter:e.handleCellMouseEnter,onClick:e.handleDropRectangle})},onMouseLeave:this.handleBoardMouseLeave})}}]),n}(c.Component),z=n(19),F=n.n(z),N=n(64),V=n(401),B=n.n(V).a,L=n(403),W=n.n(L).a,H=n(405),_=n.n(H).a,q=n(407),Y=n.n(q).a,G=n(349),U=n(61),X=n(412),J=n.n(X).a,Q=n(63),K=s.a.createContext("players-names"),Z=n(413),$=n.n(Z).a,ee=n(351),te=n(352);function ne(){var e=Object(p.a)(["\n  && {\n    text-align: center;\n  }\n"]);return ne=function(){return e},e}function ae(){var e=Object(p.a)(["\n  && {\n    background-color: ",";\n  }\n"]);return ae=function(){return e},e}function re(){var e=Object(p.a)(["\n  && {\n    margin-right: 8px;\n    margin-left: 8px;\n  }\n"]);return re=function(){return e},e}var oe=Object(f.c)(ee.a)(re()),le=Object(f.c)($)(ae(),(function(e){return Object(d.isPlayer1)(e.player)?e.theme.colors.player1:e.theme.colors.player2})),ie=Object(f.c)(te.a)(ne()),ce=function(e){var t=e.isCurrent,n=e.player,a=e.allCellsCount,r=e.playerCellsCount,o=e.readOnly,l=e.onSkipTurn;return s.a.createElement(K.Consumer,null,(function(e){var i=n&&e[n];return s.a.createElement(oe,null,s.a.createElement(G.a,{avatar:s.a.createElement(le,{player:n},i&&i.charAt(0)),title:i,subheader:Number.isInteger(r)&&Number.isInteger(a)&&F.a.get("player_controls.player_stats",{playerCellsCount:r,allCellsCount:a})}),t&&s.a.createElement(ie,null,s.a.createElement(Q.a,{title:F.a.get("player_controls.skip_turn")},s.a.createElement(U.a,{disabled:o,onClick:l},s.a.createElement(J,null)))))}))},se=n(418),ue=n.n(se);function he(){var e=Object(p.a)(["\n  display: block;\n  margin: 10px auto;\n  width: 20%;\n"]);return he=function(){return e},e}var de=f.c.img(he()),pe=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var a=arguments.length,o=new Array(a),l=0;l<a;l++)o[l]=arguments[l];return(e=t.call.apply(t,[this].concat(o))).state={open:!1},e.handleOpen=function(){e.setState({open:!0})},e.handleClose=function(){e.setState({open:!1})},e.handleGoHome=function(){window.open("".concat("/territories","/"),"_self")},e}return Object(o.a)(n,[{key:"componentDidMount",value:function(){this.props.gameover&&this.handleOpen()}},{key:"componentDidUpdate",value:function(e){this.props.gameover&&e.gameover!==this.props.gameover&&this.handleOpen()}},{key:"render",value:function(){var e=this.state.open,t=this.props.gameover;return s.a.createElement(B,{open:e,fullWidth:!0,onClose:this.handleClose},t&&s.a.createElement(Y,null,t.winner?F.a.get("congratulations.winner"):F.a.get("congratulations.draw")),t&&s.a.createElement(_,null,t.winner&&s.a.createElement(ce,{player:t.winner}),s.a.createElement(de,{src:ue.a,alt:F.a.get("congratulations.image")})),s.a.createElement(W,null,s.a.createElement(N.a,{variant:"contained",color:"primary",onClick:this.handleClose},F.a.get("congratulations.ok"))))}}]),n}(c.Component),fe=n(60),me=n(42),ve=n(328),ge=n(353),ye=n(419),be=n.n(ye),we=function(){function e(){Object(r.a)(this,e),this.world=null}return Object(o.a)(e,[{key:"setWorld",value:function(e){this.world=e,this.diceBodyMaterial=new ge.Material,this.floorBodyMaterial=new ge.Material,this.barrierBodyMaterial=new ge.Material,e.addContactMaterial(new ge.ContactMaterial(this.floorBodyMaterial,this.diceBodyMaterial,{friction:.01,restitution:.5})),e.addContactMaterial(new ge.ContactMaterial(this.barrierBodyMaterial,this.diceBodyMaterial,{friction:0,restitution:1})),e.addContactMaterial(new ge.ContactMaterial(this.diceBodyMaterial,this.diceBodyMaterial,{friction:0,restitution:.5}))}},{key:"prepareValues",value:function(e){for(var t=0;t<e.length;t++)if(e[t].value<1||e[t].dice.values<e[t].value)throw new Error("Cannot throw die to value "+e[t].value+", because it has only "+e[t].dice.values+" sides.");for(var n=0;n<e.length;n++)e[n].dice.simulationRunning=!0,e[n].vectors=e[n].dice.getCurrentVectors(),e[n].stableCount=0;this.world.addEventListener("postStep",(function t(){for(var n=!0,a=0;a<e.length;a++)e[a].dice.isFinished()?e[a].stableCount++:e[a].stableCount=0,e[a].stableCount<50&&(n=!1);if(n){console.log("all stable"),Ee.world.removeEventListener("postStep",t);for(var r=0;r<e.length;r++)e[r].dice.shiftUpperValue(e[r].value),e[r].dice.setVectors(e[r].vectors),e[r].dice.simulationRunning=!1}else Ee.world.step(Ee.world.dt)}))}}]),e}(),xe=function(){function e(t){Object(r.a)(this,e),t=this.setDefaults(t,{size:100,fontColor:"#000000",backColor:"#ffffff"}),this.object=null,this.size=t.size,this.invertUpside=!1,this.materialOptions={specular:1515554,color:15790320,shininess:40,flatShading:ve.FlatShading},this.labelColor=t.fontColor,this.diceColor=t.backColor}return Object(o.a)(e,[{key:"setDefaults",value:function(e,t){for(var n in e=e||{},t)t.hasOwnProperty(n)&&(n in e||(e[n]=t[n]));return e}},{key:"emulateThrow",value:function(e){var t=this,n=0;Ee.world.addEventListener("postStep",(function a(){t.isFinished()?50===++n&&(Ee.world.removeEventListener("postStep",a),e(t.getUpsideValue())):n=0,Ee.world.step(Ee.world.dt)}))}},{key:"isFinished",value:function(){var e=this.object.body.angularVelocity,t=this.object.body.velocity;return Math.abs(e.x)<1&&Math.abs(e.y)<1&&Math.abs(e.z)<1&&Math.abs(t.x)<1&&Math.abs(t.y)<1&&Math.abs(t.z)<1}},{key:"getUpsideValue",value:function(){for(var e,t=new ve.Vector3(0,this.invertUpside?-1:1),n=2*Math.PI,a=0;a<this.object.geometry.faces.length;++a){var r=this.object.geometry.faces[a];if(0!==r.materialIndex){var o=r.normal.clone().applyQuaternion(this.object.body.quaternion).angleTo(t);o<n&&(n=o,e=r)}}return e.materialIndex-1}},{key:"getCurrentVectors",value:function(){return{position:this.object.body.position.clone(),quaternion:this.object.body.quaternion.clone(),velocity:this.object.body.velocity.clone(),angularVelocity:this.object.body.angularVelocity.clone()}}},{key:"setVectors",value:function(e){this.object.body.position=e.position,this.object.body.quaternion=e.quaternion,this.object.body.velocity=e.velocity,this.object.body.angularVelocity=e.angularVelocity}},{key:"shiftUpperValue",value:function(e){for(var t=this.object.geometry.clone(),n=this.getUpsideValue(),a=0,r=t.faces.length;a<r;++a){var o=t.faces[a].materialIndex;if(0!==o){for(o+=e-n-1;o>this.values;)o-=this.values;for(;o<1;)o+=this.values;t.faces[a].materialIndex=o+1}}this.object.geometry=t}},{key:"getChamferGeometry",value:function(e,t,n){for(var a=[],r=[],o=new Array(e.length),l=0;l<e.length;++l)o[l]=[];for(var i=0;i<t.length;++i){for(var c=t[i],s=c.length-1,u=new ve.Vector3,h=new Array(s),d=0;d<s;++d){var p=e[c[d]].clone();u.add(p),o[c[d]].push(h[d]=a.push(p)-1)}u.divideScalar(s);for(var f=0;f<s;++f){var m=a[h[f]];m.subVectors(m,u).multiplyScalar(n).addVectors(m,u)}h.push(c[s]),r.push(h)}for(var v=0;v<t.length-1;++v)for(var g=v+1;g<t.length;++g){for(var y=[],b=-1,w=0;w<t[v].length-1;++w){var x=t[g].indexOf(t[v][w]);x>=0&&x<t[g].length-1&&(b>=0&&w!==b+1?y.unshift([v,w],[g,x]):y.push([v,w],[g,x]),b=w)}4===y.length&&r.push([r[y[0][0]][y[0][1]],r[y[1][0]][y[1][1]],r[y[3][0]][y[3][1]],r[y[2][0]][y[2][1]],-1])}for(var C=0;C<o.length;++C){for(var j=o[C],O=[j[0]],M=j.length-1;M;){for(var I=t.length;I<r.length;++I){var R=r[I].indexOf(O[O.length-1]);if(R>=0&&R<4){-1===--R&&(R=3);var E=r[I][R];if(j.indexOf(E)>=0){O.push(E);break}}}--M}O.push(-1),r.push(O)}return{vectors:a,faces:r}}},{key:"makeGeometry",value:function(e,t,n,a,r){for(var o=new ve.Geometry,l=0;l<e.length;++l){var i=e[l].multiplyScalar(n);i.index=o.vertices.push(i)-1}for(var c=0;c<t.length;++c)for(var s=t[c],u=s.length-1,h=2*Math.PI/u,d=0;d<u-2;++d)o.faces.push(new ve.Face3(s[0],s[d+1],s[d+2],[o.vertices[s[0]],o.vertices[s[d+1]],o.vertices[s[d+2]]],0,s[u]+1)),o.faceVertexUvs[0].push([new ve.Vector2((Math.cos(r)+1+a)/2/(1+a),(Math.sin(r)+1+a)/2/(1+a)),new ve.Vector2((Math.cos(h*(d+1)+r)+1+a)/2/(1+a),(Math.sin(h*(d+1)+r)+1+a)/2/(1+a)),new ve.Vector2((Math.cos(h*(d+2)+r)+1+a)/2/(1+a),(Math.sin(h*(d+2)+r)+1+a)/2/(1+a))]);return o.computeFaceNormals(),o.boundingSphere=new ve.Sphere(new ve.Vector3,n),o}},{key:"createShape",value:function(e,t,n){for(var a=new Array(e.length),r=new Array(t.length),o=0;o<e.length;++o){var l=e[o];a[o]=new ge.Vec3(l.x*n,l.y*n,l.z*n)}for(var i=0;i<t.length;++i)r[i]=t[i].slice(0,t[i].length-1);return new ge.ConvexPolyhedron(a,r)}},{key:"getGeometry",value:function(){for(var e=this.size*this.scaleFactor,t=new Array(this.vertices.length),n=0;n<this.vertices.length;++n)t[n]=(new ve.Vector3).fromArray(this.vertices[n]).normalize();var a=this.getChamferGeometry(t,this.faces,this.chamfer),r=this.makeGeometry(a.vectors,a.faces,e,this.tab,this.af);return r.cannon_shape=this.createShape(t,this.faces,e),r}},{key:"calculateTextureSize",value:function(e){return Math.max(128,Math.pow(2,Math.floor(Math.log(e)/Math.log(2))))}},{key:"createTextTexture",value:function(e,t,n){var a=document.createElement("canvas"),r=a.getContext("2d"),o=2*this.calculateTextureSize(this.size/2+this.size*this.textMargin);a.width=a.height=o,r.font=o/(1+2*this.textMargin)+"pt Arial",r.fillStyle=n,r.fillRect(0,0,a.width,a.height),r.textAlign="center",r.textBaseline="middle",r.fillStyle=t,r.fillText(e,a.width/2,a.height/2);var l=new ve.Texture(a);return l.needsUpdate=!0,l}},{key:"getMaterials",value:function(){for(var e=[],t=0;t<this.faceTexts.length;++t){var n=null;n=this.customTextTextureFunction?this.customTextTextureFunction(this.faceTexts[t],this.labelColor,this.diceColor):this.createTextTexture(this.faceTexts[t],this.labelColor,this.diceColor),e.push(new ve.MeshPhongMaterial(Object.assign({},this.materialOptions,{map:n})))}return e}},{key:"getObject",value:function(){return this.object}},{key:"create",value:function(){if(!Ee.world)throw new Error("You must call DiceManager.setWorld(world) first.");return this.object=new ve.Mesh(this.getGeometry(),this.getMaterials()),this.object.reveiceShadow=!0,this.object.castShadow=!0,this.object.diceObject=this,this.object.body=new ge.Body({mass:this.mass,shape:this.object.geometry.cannon_shape,material:Ee.diceBodyMaterial}),this.object.body.linearDamping=.1,this.object.body.angularDamping=.1,Ee.world.add(this.object.body),this.object}},{key:"updateMeshFromBody",value:function(){this.simulationRunning||(this.object.position.copy(this.object.body.position),this.object.quaternion.copy(this.object.body.quaternion))}},{key:"updateBodyFromMesh",value:function(){this.object.body.position.copy(this.object.position),this.object.body.quaternion.copy(this.object.quaternion)}}]),e}(),Ce=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(e){var a;return Object(r.a)(this,n),(a=t.call(this,e)).tab=-.1,a.af=7*Math.PI/6,a.chamfer=.96,a.vertices=[[1,1,1],[-1,-1,1],[-1,1,-1],[1,-1,-1]],a.faces=[[1,0,2,1],[0,1,3,2],[0,3,2,3],[1,2,3,4]],a.scaleFactor=1.2,a.values=4,a.faceTexts=[[],["0","0","0"],["2","4","3"],["1","3","4"],["2","1","4"],["1","2","3"]],a.customTextTextureFunction=function(e,t,n){var a=document.createElement("canvas"),r=a.getContext("2d"),o=2*this.calculateTextureSize(this.size/2+2*this.size);for(var l in a.width=a.height=o,r.font=o/5+"pt Arial",r.fillStyle=n,r.fillRect(0,0,a.width,a.height),r.textAlign="center",r.textBaseline="middle",r.fillStyle=t,e)r.fillText(e[l],a.width/2,a.height/2-.3*o),r.translate(a.width/2,a.height/2),r.rotate(2*Math.PI/3),r.translate(-a.width/2,-a.height/2);var i=new ve.Texture(a);return i.needsUpdate=!0,i},a.mass=300,a.inertia=5,a.invertUpside=!0,a.create(),a}return n}(xe),je=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(e){var a;return Object(r.a)(this,n),(a=t.call(this,e)).tab=.1,a.af=Math.PI/4,a.chamfer=.96,a.vertices=[[-1,-1,-1],[1,-1,-1],[1,1,-1],[-1,1,-1],[-1,-1,1],[1,-1,1],[1,1,1],[-1,1,1]],a.faces=[[0,3,2,1,1],[1,2,6,5,2],[0,1,5,4,3],[3,7,6,2,4],[0,4,7,3,5],[4,5,6,7,6]],a.scaleFactor=.9,a.values=6,a.faceTexts=[" ","0","1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19","20"],a.textMargin=1,a.mass=300,a.inertia=13,a.create(),a}return n}(xe),Oe=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(e){var a;return Object(r.a)(this,n),(a=t.call(this,e)).tab=0,a.af=-Math.PI/4/2,a.chamfer=.965,a.vertices=[[1,0,0],[-1,0,0],[0,1,0],[0,-1,0],[0,0,1],[0,0,-1]],a.faces=[[0,2,4,1],[0,4,3,2],[0,3,5,3],[0,5,2,4],[1,3,4,5],[1,4,2,6],[1,2,5,7],[1,5,3,8]],a.scaleFactor=1,a.values=8,a.faceTexts=[" ","0","1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19","20"],a.textMargin=1.2,a.mass=340,a.inertia=10,a.create(),a}return n}(xe),Me=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(e){var a;Object(r.a)(this,n),(a=t.call(this,e)).tab=0,a.af=6*Math.PI/5,a.chamfer=.945,a.vertices=[],a.faces=[[5,7,11,0],[4,2,10,1],[1,3,11,2],[0,8,10,3],[7,9,11,4],[8,6,10,5],[9,1,11,6],[2,0,10,7],[3,5,11,8],[6,4,10,9],[1,0,2,-1],[1,2,3,-1],[3,2,4,-1],[3,4,5,-1],[5,4,6,-1],[5,6,7,-1],[7,6,8,-1],[7,8,9,-1],[9,8,0,-1],[9,0,1,-1]];for(var o=0,l=0;o<10;++o,l+=2*Math.PI/10)a.vertices.push([Math.cos(l),Math.sin(l),.105*(o%2?1:-1)]);return a.vertices.push([0,0,-1]),a.vertices.push([0,0,1]),a.scaleFactor=.9,a.values=10,a.faceTexts=[" ","0","1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19","20"],a.textMargin=1,a.mass=350,a.inertia=9,a.create(),a}return n}(xe),Ie=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(e){var a;Object(r.a)(this,n),a=t.call(this,e);var o=(1+Math.sqrt(5))/2,l=1/o;return a.tab=.2,a.af=-Math.PI/4/2,a.chamfer=.968,a.vertices=[[0,l,o],[0,l,-o],[0,-l,o],[0,-l,-o],[o,0,l],[o,0,-l],[-o,0,l],[-o,0,-l],[l,o,0],[l,-o,0],[-l,o,0],[-l,-o,0],[1,1,1],[1,1,-1],[1,-1,1],[1,-1,-1],[-1,1,1],[-1,1,-1],[-1,-1,1],[-1,-1,-1]],a.faces=[[2,14,4,12,0,1],[15,9,11,19,3,2],[16,10,17,7,6,3],[6,7,19,11,18,4],[6,18,2,0,16,5],[18,11,9,14,2,6],[1,17,10,8,13,7],[1,13,5,15,3,8],[13,8,12,4,5,9],[5,4,14,9,15,10],[0,12,8,10,16,11],[3,19,7,17,1,12]],a.scaleFactor=.9,a.values=12,a.faceTexts=[" ","0","1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19","20"],a.textMargin=1,a.mass=350,a.inertia=8,a.create(),a}return n}(xe),Re=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(e){var a;Object(r.a)(this,n),a=t.call(this,e);var o=(1+Math.sqrt(5))/2;return a.tab=-.2,a.af=-Math.PI/4/2,a.chamfer=.955,a.vertices=[[-1,o,0],[1,o,0],[-1,-o,0],[1,-o,0],[0,-1,o],[0,1,o],[0,-1,-o],[0,1,-o],[o,0,-1],[o,0,1],[-o,0,-1],[-o,0,1]],a.faces=[[0,11,5,1],[0,5,1,2],[0,1,7,3],[0,7,10,4],[0,10,11,5],[1,5,9,6],[5,11,4,7],[11,10,2,8],[10,7,6,9],[7,1,8,10],[3,9,4,11],[3,4,2,12],[3,2,6,13],[3,6,8,14],[3,8,9,15],[4,9,5,16],[2,4,11,17],[6,2,10,18],[8,6,7,19],[9,8,1,20]],a.scaleFactor=1,a.values=20,a.faceTexts=[" ","0","1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19","20"],a.textMargin=1,a.mass=400,a.inertia=6,a.create(),a}return n}(xe),Ee=new we,ke="D4",De="D6",Se="D8",Te="D10",Pe="D12",Ae="D20",ze=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var o=arguments.length,l=new Array(o),i=0;i<o;i++)l[i]=arguments[i];return(e=t.call.apply(t,[this].concat(l))).generateDiceModels=function(e){return e.map((function(e){var t=e.type,n=e.size||4;return t===ke?new Ce(Object(a.a)({size:n},e)):t===De?new je(Object(a.a)({size:n},e)):t===Se?new Oe(Object(a.a)({size:n},e)):t===Te?new Me(Object(a.a)({size:n},e)):t===Pe?new Ie(Object(a.a)({size:n},e)):t===Ae?new Re(Object(a.a)({size:n},e)):new xe(Object(a.a)({size:n},e))}))},e.prepareDicesValues=function(t){var n=e.diceModels.map((function(e,n){var a=e.size,r=e.getObject();r.position.x=-15-n%3*a,r.position.y=2+Math.floor(n/3)*a,r.position.z=n%3*a-15,r.quaternion.x=(90*Math.random()-45)*Math.PI/180,r.quaternion.z=(90*Math.random()-45)*Math.PI/180,e.updateBodyFromMesh();var o=20*Math.random(),l=5*Math.random();return r.body.velocity.set(25+l,40+o,15+l),{dice:e,value:t[n]}}));Ee.prepareValues(n)},e.start=function(){e.frameId||(e.frameId=requestAnimationFrame(e.animate))},e.stop=function(){cancelAnimationFrame(e.frameId)},e.animate=function(){e.updatePhysics(),e.renderScene(),e.controls.update(),e.frameId=window.requestAnimationFrame(e.animate)},e.updatePhysics=function(){e.world.step(1/60),e.diceModels.forEach((function(e){return e.updateMeshFromBody()}))},e.renderScene=function(){e.renderer.render(e.scene,e.camera)},e}return Object(o.a)(n,[{key:"componentDidMount",value:function(){var e=this;if(!Array.isArray(this.props.dices))throw new Error("Required argument 'dices' is missing. It has to be an array of dice configs like [{ type: 'D6', backColor: 'red', value: 4 }]");this.scene=new ve.Scene,this.scene.background="white";var t=this.mount.clientWidth,n=this.mount.clientHeight,a=t/n;this.camera=new ve.PerspectiveCamera(45,a,.1,1e3),this.camera.position.set(0,30,30),this.scene.add(this.camera),this.renderer=new ve.WebGLRenderer({antialias:!0}),this.renderer.setSize(t,n),this.renderer.shadowMap.enabled=!0,this.mount.appendChild(this.renderer.domElement),this.controls=new be.a(this.camera,this.renderer.domElement);var r=new ve.AmbientLight("#ffffff",.3);this.scene.add(r);var o=new ve.DirectionalLight("#ffffff",.5);o.position.x=-1e3,o.position.y=1e3,o.position.z=1e3,this.scene.add(o);var l=new ve.SpotLight(15720405,1.3);l.position.y=100,l.target.position.set(0,0,0),l.castShadow=!0,l.shadow.camera.near=50,l.shadow.camera.far=110,l.shadow.mapSize.width=1024,l.shadow.mapSize.height=1024,this.scene.add(l),this.world=new ge.World,this.world.gravity.set(0,-196.4,0),this.world.broadphase=new ge.NaiveBroadphase,this.world.solver.iterations=16,Ee.setWorld(this.world);var i=new ge.Body({mass:0,shape:new ge.Plane,material:Ee.floorBodyMaterial});i.quaternion.setFromAxisAngle(new ge.Vec3(1,0,0),-Math.PI/2),this.world.add(i),this.diceModels=this.generateDiceModels(this.props.dices),this.diceModels.forEach((function(t){return e.scene.add(t.getObject())})),this.prepareDicesValues(this.props.dices.map((function(e){return e.value}))),this.start()}},{key:"componentWillUnmount",value:function(){this.stop(),this.mount.removeChild(this.renderer.domElement)}},{key:"render",value:function(){var e=this;return s.a.createElement("div",{className:this.props.className,style:Object(a.a)({width:this.props.width,height:this.props.height||"300px"},this.props.style),ref:function(t){e.mount=t}})}}]),n}(c.Component),Fe=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var a=arguments.length,o=new Array(a),l=0;l<a;l++)o[l]=arguments[l];return(e=t.call.apply(t,[this].concat(o))).state={open:!1},e}return Object(o.a)(n,[{key:"componentDidUpdate",value:function(e){var t=this,n=this.props.rollingDices;n&&n!==e.rollingDices&&(this.setState({open:!0}),setTimeout((function(){t.setState({open:!1}),t.props.onFinishRoll()}),3e3))}},{key:"render",value:function(){var e=this.props.rollingDices,t=this.state.open;return s.a.createElement(c.Fragment,null,s.a.createElement(B,{open:t,fullWidth:!0},t&&e&&s.a.createElement(ze,{dices:[{type:De,backColor:"red",fontColor:"white",value:e[0]},{type:De,backColor:"blue",fontColor:"white",value:e[1]}]})))}}]),n}(c.Component),Ne=n(420),Ve=n.n(Ne).a,Be=n(62),Le=n.n(Be).a;function We(){var e=Object(p.a)(["\n  height: 100%;\n"]);return We=function(){return e},e}function He(){var e=Object(p.a)(["\n  && {\n    height: ",";\n    width: ",";\n  }\n"]);return He=function(){return e},e}var _e=Object(f.c)(Le)(He(),(function(e){return e.height}),(function(e){return e.width})),qe=Object(f.c)(fe.a)(We()),Ye=function(e){var t=e.cellRadius,n=e.currentPlayer,a=e.dices,r=e.rollingDices,o=e.allCellsCount,l=e.occupiedCounters,i=e.readOnly,c=e.onStartRollDices,u=e.onFinishRollDices,h=e.onRotateRectangle,p=e.onSkipTurn;return s.a.createElement(fe.a,null,s.a.createElement(fe.b,null,s.a.createElement(ce,{player:d.PLAYER_1,isCurrent:d.isPlayer1(n),allCellsCount:o,playerCellsCount:l[d.PLAYER_1],readOnly:i,onSkipTurn:p})),s.a.createElement(fe.b,{center:!0},s.a.createElement("div",null,s.a.createElement(_e,{height:"".concat(25*t,"px"),width:"".concat(25*t,"px")},a&&0!==a[0]?s.a.createElement(qe,{column:!0,center:!0,alignItems:"center"},s.a.createElement(fe.b,null,s.a.createElement(Q.a,{title:"Rotate rectangle"},s.a.createElement(U.a,{disabled:i,size:"small",onClick:h},s.a.createElement(Ve,null)))),s.a.createElement(fe.b,{center:!0},s.a.createElement(P,{rows:Array(a[0]).fill(Array(a[1]).fill()),cellRadius:t}))):s.a.createElement(qe,{column:!0,center:!0,alignItems:"center"},s.a.createElement(N.a,{disabled:i,color:d.isPlayer1(n)?"primary":"secondary",size:"large",variant:"contained",onClick:c},F.a.get("player_controls.roll_dices")))),s.a.createElement(Fe,{rollingDices:r,onFinishRoll:u}))),s.a.createElement(fe.b,null,s.a.createElement(ce,{player:d.PLAYER_2,isCurrent:d.isPlayer2(n),allCellsCount:o,playerCellsCount:l[d.PLAYER_2],readOnly:i,onSkipTurn:p})))};function Ge(){var e=Object(p.a)(["\n  /* Bottom margin from board */\n  margin-bottom: 24px;\n"]);return Ge=function(){return e},e}var Ue=Object(f.c)(fe.b)(Ge()),Xe=function(e){Object(l.a)(n,e);var t=Object(i.a)(n);function n(){var e;Object(r.a)(this,n);for(var a=arguments.length,o=new Array(a),l=0;l<a;l++)o[l]=arguments[l];return(e=t.call.apply(t,[this].concat(o))).state={isLoadingNames:!1,playersNames:{[d.PLAYER_1]:"Alice",[d.PLAYER_2]:"Bob"}},e.handleStartRollDices=function(){e.props.moves.startRollDices()},e.handleFinishRollDices=function(){var t=e.props,n=t.isMultiplayer,a=t.playerID,r=t.ctx.currentPlayer;n&&a!==r||e.props.moves.finishRollDices()},e.handleRotateRectangle=function(){e.props.moves.switchDices()},e.handleDropRectangle=function(t){var n=t.rowIndex,a=t.columnIndex,r=t.rectangleHeight,o=t.rectangleWidth;e.props.moves.dropRectangle(n,a,r,o),e.handleEndTurn()},e.handleEndTurn=function(){e.props.moves.clearDices(),e.props.events.endTurn()},e}return Object(o.a)(n,[{key:"componentDidMount",value:function(){var e=this,t=this.props,n=t.ai,r=t.isMultiplayer,o=t.gameID;n?this.setState({playersNames:Object(a.a)(Object(a.a)({},this.state.playersNames),{},{[n.playerID]:n.name||"Bot"})}):r&&o&&(this.setState({isLoadingNames:!0}),h.a.get("".concat("https://territories-backend.herokuapp.com","/games/territories")).then((function(t){var n=t.data&&t.data.gameInstances.find((function(e){return e.gameID===o}));n&&e.setState({playersNames:n.players.reduce((function(e,t){return e["".concat(t.id)]=t.name,e}),{})})})).finally((function(){return e.setState({isLoadingNames:!1})})))}},{key:"componentDidUpdate",value:function(e){var t=this,n=this.props.ai;if(n){var a=this.props.ctx.currentPlayer;if(n.playerID===a){e.ctx.currentPlayer!==a&&setTimeout((function(){t.handleStartRollDices()}),500);var r=this.props.G,o=r.dices,l=r.board,i=r.allCellsCount,c=r.occupiedCounters,s=this.props.ctx.turn;if(o&&e.G.dices&&e.G.dices!==o&&0!==o[0]&&o[0]!==e.G.dices[1]&&o[1]!==e.G.dices[0]){var u=o[0],h=o[1],d=n.guessCellToDropRectangle({turn:s,currentPlayer:a,rectangleHeight:u,rectangleWidth:h,rows:l,emptyCellsCount:i-c[0]-c[1]});-1===d.rowIndex||-1===d.columnIndex?setTimeout((function(){t.handleEndTurn()}),1e3):d.rectangleHeight!==u?setTimeout((function(){t.handleRotateRectangle(),setTimeout((function(){t.handleDropRectangle(d)}),1e3)}),1e3):setTimeout((function(){t.handleDropRectangle(d)}),1e3)}}}}},{key:"render",value:function(){var e=this.props,t=e.G,n=t.board,a=t.dices,r=t.rollingDices,o=t.allCellsCount,l=t.occupiedCounters,i=e.ctx.currentPlayer,c=e.isActive,u=e.ai,h=this.state,p=h.isLoadingNames,f=h.playersNames;if(p)return s.a.createElement(me.a,{color:"primary"});var m=!c||u&&u.playerID===i;return s.a.createElement(K.Provider,{value:f},s.a.createElement(fe.a,{column:!0},s.a.createElement(Ue,{center:!0},s.a.createElement(Ye,{cellRadius:10,dices:a,rollingDices:r,currentPlayer:i,allCellsCount:o,occupiedCounters:l,readOnly:m,onStartRollDices:this.handleStartRollDices,onFinishRollDices:this.handleFinishRollDices,onRotateRectangle:this.handleRotateRectangle,onSkipTurn:this.handleEndTurn})),s.a.createElement(fe.b,{center:!0},s.a.createElement(A,{disabled:m||!a||0===a[0],cellRadius:10,rows:n,rectangleHeight:a?a[0]:0,rectangleWidth:a?a[1]:0,currentPlayer:i,onDropRectangle:this.handleDropRectangle})),s.a.createElement(pe,{gameover:Object(d.selectGameover)({allCellsCount:o,occupiedCounters:l})})))}}]),n}(c.Component);t.a=Xe},349:function(e,t,n){"use strict";var a=n(410),r=n.n(a);t.a=r.a},351:function(e,t,n){"use strict";var a=n(414),r=n.n(a);t.a=r.a},352:function(e,t,n){"use strict";var a=n(416),r=n.n(a);t.a=r.a},394:function(e,t){},418:function(e,t,n){e.exports=n.p+"static/media/congratulations.83861842.gif"}}]);
//# sourceMappingURL=1.30a8cb2f.chunk.js.map
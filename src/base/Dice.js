import React, { Component } from "react";
import * as THREE from "three";
import * as CANNON from "cannon";
import OrbitControls from "orbit-controls-es6";

import { DiceManager, DiceD6 } from "../libs/dice";

class ThreeScene extends Component {
  componentDidMount() {
    // const width = this.mount.clientWidth;
    // const height = this.mount.clientHeight;
    // //ADD SCENE
    // this.scene = new THREE.Scene();
    // //ADD CAMERA
    // this.camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
    // this.camera.position.z = 4;
    // //ADD RENDERER
    // this.renderer = new THREE.WebGLRenderer({ antialias: true });
    // this.renderer.setClearColor("#000000");
    // this.renderer.setSize(width, height);
    // this.mount.appendChild(this.renderer.domElement);
    // //ADD CUBE
    // const geometry = new THREE.BoxGeometry(1, 1, 1);
    // const material = new THREE.MeshBasicMaterial({ color: "#433F81" });
    // this.cube = new THREE.Mesh(geometry, material);
    // this.scene.add(this.cube);
    // this.start();
    // SCENE
    this.scene = new THREE.Scene();
    // CAMERA
    const SCREEN_WIDTH = window.innerWidth;
    const SCREEN_HEIGHT = window.innerHeight;
    const VIEW_ANGLE = 45;
    const ASPECT = SCREEN_WIDTH / SCREEN_HEIGHT;
    const NEAR = 0.01;
    const FAR = 20000;
    this.camera = new THREE.PerspectiveCamera(VIEW_ANGLE, ASPECT, NEAR, FAR);
    this.scene.add(this.camera);
    this.camera.position.set(0, 30, 30);
    // RENDERER
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(SCREEN_WIDTH, SCREEN_HEIGHT);
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;

    this.mount.appendChild(this.renderer.domElement);

    // EVENTS
    // CONTROLS
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);

    const ambient = new THREE.AmbientLight("#ffffff", 0.3);
    this.scene.add(ambient);
    const directionalLight = new THREE.DirectionalLight("#ffffff", 0.5);
    directionalLight.position.x = -1000;
    directionalLight.position.y = 1000;
    directionalLight.position.z = 1000;
    this.scene.add(directionalLight);
    const light = new THREE.SpotLight(0xefdfd5, 1.3);
    light.position.y = 100;
    light.target.position.set(0, 0, 0);
    light.castShadow = true;
    light.shadow.camera.near = 50;
    light.shadow.camera.far = 110;
    light.shadow.mapSize.width = 1024;
    light.shadow.mapSize.height = 1024;
    this.scene.add(light);
    // FLOOR
    const floorMaterial = new THREE.MeshPhongMaterial({
      color: "#00aa00",
      side: THREE.DoubleSide
    });
    const floorGeometry = new THREE.PlaneGeometry(30, 30, 10, 10);
    const floor = new THREE.Mesh(floorGeometry, floorMaterial);
    floor.receiveShadow = true;
    floor.rotation.x = Math.PI / 2;
    this.scene.add(floor);
    this.scene.fog = new THREE.FogExp2(0x9999ff, 0.00025);
    ////////////
    // CUSTOM //
    ////////////
    this.world = new CANNON.World();
    this.world.gravity.set(0, -9.82 * 20, 0);
    this.world.broadphase = new CANNON.NaiveBroadphase();
    this.world.solver.iterations = 16;
    DiceManager.setWorld(this.world);
    //Floor
    const floorBody = new CANNON.Body({
      mass: 0,
      shape: new CANNON.Plane(),
      material: DiceManager.floorBodyMaterial
    });
    floorBody.quaternion.setFromAxisAngle(
      new CANNON.Vec3(1, 0, 0),
      -Math.PI / 2
    );
    this.world.add(floorBody);
    //Walls

    this.dice = [];
    const colors = ["#ff0000", "#ffff00"];
    for (let i = 0; i < 2; i++) {
      const die = new DiceD6({ size: 1.5, backColor: colors[i] });
      this.scene.add(die.getObject());
      this.dice.push(die);
    }

    setInterval(this.randomDiceThrow, 3000);
    this.randomDiceThrow();
    this.start();
  }

  componentWillUnmount() {
    this.stop();
    this.mount.removeChild(this.renderer.domElement);
  }

  start = () => {
    if (!this.frameId) {
      this.frameId = requestAnimationFrame(this.animate);
    }
  };

  stop = () => {
    cancelAnimationFrame(this.frameId);
  };

  randomDiceThrow = () => {
    const diceValues = [];
    for (var i = 0; i < this.dice.length; i++) {
      let yRand = Math.random() * 20;
      this.dice[i].getObject().position.x = -15 - (i % 3) * 1.5;
      this.dice[i].getObject().position.y = 2 + Math.floor(i / 3) * 1.5;
      this.dice[i].getObject().position.z = -15 + (i % 3) * 1.5;
      this.dice[i].getObject().quaternion.x =
        ((Math.random() * 90 - 45) * Math.PI) / 180;
      this.dice[i].getObject().quaternion.z =
        ((Math.random() * 90 - 45) * Math.PI) / 180;
      this.dice[i].updateBodyFromMesh();
      let rand = Math.random() * 5;
      this.dice[i]
        .getObject()
        .body.velocity.set(25 + rand, 40 + yRand, 15 + rand);
      this.dice[i]
        .getObject()
        .body.angularVelocity.set(
          20 * Math.random() - 10,
          20 * Math.random() - 10,
          20 * Math.random() - 10
        );
      diceValues.push({ dice: this.dice[i], value: i + 1 });
    }

    DiceManager.prepareValues(diceValues);
  };

  animate = () => {
    this.updatePhysics();
    this.renderScene();
    this.controls.update();
    this.frameId = window.requestAnimationFrame(this.animate);
  };

  updatePhysics = () => {
    this.world.step(1.0 / 60.0);
    for (var i in this.dice) {
      this.dice[i].updateMeshFromBody();
    }
  };

  renderScene = () => {
    this.renderer.render(this.scene, this.camera);
  };

  render() {
    return (
      <div
        style={{
          position: "absolute",
          left: "0px",
          top: "0px"
        }}
        ref={mount => {
          this.mount = mount;
        }}
      />
    );
  }
}
export default ThreeScene;

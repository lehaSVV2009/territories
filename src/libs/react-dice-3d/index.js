// See:
// dice logic + old, but great ui - http://a.teall.info/dice
// dice throw - https://github.com/byWulf/threejs-dice/blob/master/examples/rolling.html
// three.js in React - https://medium.com/@colesayershapiro/using-three-js-in-react-6cb71e87bdf4
// Thanks to all the authors!!!
import React, { Component } from "react";
import * as THREE from "three";
import * as CANNON from "cannon";
import OrbitControls from "orbit-controls-es6";

import { DiceManager, DiceD6 } from "./dice";

const DICE_SIZE = 4;

class Dice extends Component {
  componentDidMount() {
    if (!Array.isArray(this.props.dices)) {
      throw new Error(
        "Required argument 'dices' is missing. It has to be an array of dice configs like [{ color: 'white' }, { color: 'red', size: 10 }]"
      );
    }

    // SCENE
    this.scene = new THREE.Scene();
    this.scene.background = "white";
    // CAMERA
    const width = this.mount.clientWidth;
    const height = this.mount.clientHeight;
    const viewAngle = 45;
    const ratio = width / height;
    const near = 0.1;
    const far = 1000;
    this.camera = new THREE.PerspectiveCamera(viewAngle, ratio, near, far);
    this.camera.position.set(0, 30, 30);
    this.scene.add(this.camera);

    // RENDERER
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(width, height);
    this.renderer.shadowMap.enabled = true;

    this.mount.appendChild(this.renderer.domElement);

    // CONTROLS
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);

    // LIGHTS
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

    ////////////
    // CUSTOM //
    ////////////
    this.world = new CANNON.World();
    this.world.gravity.set(0, -9.82 * 20, 0);
    this.world.broadphase = new CANNON.NaiveBroadphase();
    this.world.solver.iterations = 16;
    DiceManager.setWorld(this.world);

    // Floor
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

    // Dice
    this.dices = this.generateDices();

    this.throwDices();
  }

  componentWillUnmount() {
    this.stop();
    this.mount.removeChild(this.renderer.domElement);
  }

  componentDidUpdate(prevProps) {
    if (this.props.value !== prevProps.value) {
      this.dices.forEach(die => this.scene.remove(die.getObject()));
      this.dices = this.generateDices();
      this.throwDices();
    }
  }

  generateDices = () => {
    return this.props.dices.map(diceOptions => {
      // TODO add different types
      const die = new DiceD6({ size: DICE_SIZE, ...diceOptions });
      this.scene.add(die.getObject());
      return die;
    });
  };

  throwDices = () => {
    const { value } = this.props;

    if (
      !Array.isArray(value) ||
      !value.every(number => number > 0 && number < 7)
    ) {
      return;
    }

    const diceValues = [];
    for (var i = 0; i < this.dices.length; i++) {
      let yRand = Math.random() * 20;
      this.dices[i].getObject().position.x = -15 - (i % 3) * DICE_SIZE;
      this.dices[i].getObject().position.y = 2 + Math.floor(i / 3) * DICE_SIZE;
      this.dices[i].getObject().position.z = -15 + (i % 3) * DICE_SIZE;
      this.dices[i].getObject().quaternion.x =
        ((Math.random() * 90 - 45) * Math.PI) / 180;
      this.dices[i].getObject().quaternion.z =
        ((Math.random() * 90 - 45) * Math.PI) / 180;
      this.dices[i].updateBodyFromMesh();
      let rand = Math.random() * 5;
      this.dices[i]
        .getObject()
        .body.velocity.set(25 + rand, 40 + yRand, 15 + rand);
      this.dices[i]
        .getObject()
        .body.angularVelocity.set(
          20 * Math.random() - 10,
          20 * Math.random() - 10,
          20 * Math.random() - 10
        );
      diceValues.push({ dice: this.dices[i], value: value[i] });
    }

    DiceManager.prepareValues(diceValues);
    this.start();
  };

  start = () => {
    if (!this.frameId) {
      this.frameId = requestAnimationFrame(this.animate);
    }
  };

  stop = () => {
    cancelAnimationFrame(this.frameId);
  };

  animate = () => {
    this.updatePhysics();
    this.renderScene();
    this.controls.update();
    this.frameId = window.requestAnimationFrame(this.animate);
  };

  updatePhysics = () => {
    this.world.step(1.0 / 60.0);
    for (var i in this.dices) {
      this.dices[i].updateMeshFromBody();
    }
  };

  renderScene = () => {
    this.renderer.render(this.scene, this.camera);
  };

  render() {
    return (
      <div
        style={{
          width: this.props.width || "400px",
          height: this.props.height || "300px",
          ...this.props.style
        }}
        ref={mount => {
          this.mount = mount;
        }}
      />
    );
  }
}

export default Dice;

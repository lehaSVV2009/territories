import styled from "styled-components";

export const Face = styled.div`
  width: 300px;
  height: 300px;
  border: 4px solid black;
  border-radius: 10px;
  background-color: white;
  margin: 0 auto;
  margin-top: 100px;

  @media screen and (max-width: $sm) {
    margin-top: 40px;
    transform: scale(0.8);
  }
`;

export const Band = styled.div`
  width: 350px;
  height: 27px;
  border: 4px solid black;
  border-radius: 5px;
  margin-left: -29px;
  margin-top: 50px;
`;

export const White = styled.div`
  height: calc(100% / 2);
  width: 100%;
  background-color: white;
`;

export const Red = styled.div`
  height: calc(100% / 2);
  width: 100%;
  background-color: red;
`;

export const Eyes = styled.div`
  width: 128px;
  margin: 0 auto;
  padding-left: 10px;
  margin-top: 40px;
  &:before {
    content: "";
    display: inline-block;
    width: 30px;
    height: 15px;
    border: 7px solid black;
    margin-right: 20px;
    border-top-left-radius: 22px;
    border-top-right-radius: 22px;
    border-bottom: 0;
  }

  &:after {
    content: "";
    display: inline-block;
    width: 30px;
    height: 15px;
    border: 7px solid black;
    margin-left: 20px;
    border-top-left-radius: 22px;
    border-top-right-radius: 22px;
    border-bottom: 0;
  }
`;

export const Dimples = styled.div`
  width: 180px;
  margin: 0 auto;
  margin-top: 15px;

  &:before {
    content: "";
    display: inline-block;
    width: 10px;
    height: 10px;
    margin-right: 80px;
    border-radius: 50%;
    background-color: rgba(red, 0.4);
  }

  &:after {
    content: "";
    display: inline-block;
    width: 10px;
    height: 10px;
    margin-left: 80px;
    border-radius: 50%;
    background-color: rgba(red, 0.4);
  }
`;

export const Mouth = styled.div`
  width: 40px;
  height: 5px;
  border-radius: 5px;
  background-color: black;
  margin: 0 auto;
  margin-top: 25px;
`;

export const ErrorTitle = styled.h1`
  font-weight: 800;
  color: black;
  text-align: center;
  font-size: 2.5em;
  padding-top: 20px;

  @media screen and (max-width: 600px) {
    padding-left: 20px;
    padding-right: 20px;
    font-size: 2em;
  }
`;

export const ErrorDescription = styled.div`
  text-align: center;
  color: red;
  padding: 10px;
`;

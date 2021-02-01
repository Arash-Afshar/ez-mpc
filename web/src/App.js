import React, { useState } from 'react';
import './App.css';

const handleClick = (wasm) => {
  alert(wasm.add_values(3, 4));
}
const Loaded = ({ wasm }) => <button onClick={() => handleClick(wasm)}>Click me</button>;

const App = () => {
  const [wasm, setWasm] = useState(null);

  React.useEffect(() => {
    const loadWasm = async () => {
      const wasm = await import('mpc');
      setWasm(wasm);
    }
    loadWasm();
  }, []);

  return (
    <div className="App">
      <header className="App-header">
        {wasm ? (
          <Loaded wasm={wasm} />
        ) :
          <div>Loading...</div>
        }
      </header>
    </div>
  );
};

export default App;
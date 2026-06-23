import React, { useState, useEffect } from "react";
import "./App.css";

function App() {
  const [potsCount, setPotsCount] = useState(6);
  const [level, setLevel] = useState("EASY");
  const [gameStatus, setGameStatus] = useState("INIT");
  const [pots, setPots] = useState([]);
  const [potsStatus, setPotsStatus] = useState([]);
  const [userScore, setUserScore] = useState(0);
  const [compScore, setCompScore] = useState(0);
  const [currentPlayer, setCurrentPlayer] = useState("USER");
  const [firstTurn, setFirstTurn] = useState("USER");
  const [edgeIndex, setEdgeIndex] = useState([0, 5]);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (gameStatus === "PROG" && currentPlayer === "COMP") {
      setTimeout(() => {
        computerMove();
      }, 2000);
    }
  }, [gameStatus, currentPlayer]);

  const startGame = async () => {
    try {
      const response = await fetch("https://pog-api.shuttleapp.rs/api/start-game", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ potscount: potsCount }),
      });

      if (!response.ok) {
        throw new Error("Network response was not ok");
      }

      const data = await response.json();
      const initialStatus = data.pots.map((_, index) =>
        index === 0 || index === data.pots.length - 1 ? "E" : "I"
      );

      setPots(data.pots);
      setPotsStatus(initialStatus);
      setEdgeIndex([0, data.pots.length - 1]);
      setError(null);
      setGameStatus("PROG");
      setCurrentPlayer(firstTurn);
      setUserScore(0);
      setCompScore(0);
    } catch (error) {
      setError("Failed to start game. Please try again.");
      console.error("There was an error!", error);
    }
  };

  const handlePlayerMove = (index) => {
    setUserScore(userScore + pots[index]);
    updatePotsAfterMove(index, "USER");
    setCurrentPlayer("COMP");
  };

  const computerMove = async () => {
    try {
      const [start, end] = edgeIndex;
      const response = await fetch("https://pog-api.shuttleapp.rs/api/optimal-move", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ pots: pots.slice(start, end + 1), level }),
      });

      if (!response.ok) {
        throw new Error("Network response was not ok");
      }

      const data = await response.json();
      const origIndex = data.chosen_index === 0 ? start : end;

      setCompScore(compScore + pots[origIndex]);
      updatePotsAfterMove(origIndex, "COMP");
    } catch (error) {
      setError("Failed to perform computer move. Please try again.");
      console.error("There was an error!", error);
    }
  };

  const updatePotsAfterMove = (index, player) => {
    const newPotsStatus = [...potsStatus];
    newPotsStatus[index] = player === "USER" ? "P" : "C";

    if (index > 0 && newPotsStatus[index - 1] === "I") {
      newPotsStatus[index - 1] = "E";
    }
    if (index < pots.length - 1 && newPotsStatus[index + 1] === "I") {
      newPotsStatus[index + 1] = "E";
    }

    setPotsStatus(newPotsStatus);

    if (index === edgeIndex[0]) {
      setEdgeIndex([index + 1, edgeIndex[1]]);
    } else if (index === edgeIndex[1]) {
      setEdgeIndex([edgeIndex[0], index - 1]);
    }

    if (newPotsStatus.every((status) => status === "P" || status === "C")) {
      setGameStatus("DONE");
    } else {
      setCurrentPlayer(currentPlayer === "USER" ? "COMP" : "USER");
    }
  };

  return (
    <div className="App">
      <div className="Header">
        <p>Pots of Gold</p>
      </div>
      {gameStatus === "INIT" ? (
        <div className="game-options">
          <div className="option-group">
            <h2>Pots Count:</h2>
            {[6, 8, 10, 12].map((count) => (
              <label key={count}>
                <input
                  type="radio"
                  value={count}
                  checked={potsCount === count}
                  onChange={() => setPotsCount(count)}
                />
                {count}
              </label>
            ))}
          </div>
          <div className="option-group">
            <h2>Difficulty:    </h2>
            {["EASY", "MEDIUM", "HARD"].map((lvl) => (
              <label key={lvl}>
                <input
                  type="radio"
                  value={lvl}
                  checked={level === lvl}
                  onChange={() => setLevel(lvl)}
                />
                {lvl}
              </label>
            ))}
          </div>
          <div className="option-group">
            <h2>First Turn:</h2>
            <label>
              <input
                type="radio"
                value="USER"
                checked={firstTurn === "USER"}
                onChange={() => setFirstTurn("USER")}
              />
              USER
            </label>
            <label>
              <input
                type="radio"
                value="COMP"
                checked={firstTurn === "COMP"}
                onChange={() => setFirstTurn("COMP")}
              />
              COMPUTER
            </label>
          </div>
          <button className="option-group" onClick={startGame}>
            Start
          </button>
          {error && <p className="error">{error}</p>}
        </div>
      ) : (
        <div className="game-board">
          {gameStatus === "DONE" ? (
            <div className="game-over">
              <h2>Game Over!!!</h2>
              <p>Player Score: {userScore}</p>
              <p>Computer Score: {compScore}</p>
              <p>Winner: {userScore > compScore ? "USER" : "COMPUTER"}</p>
              <button onClick={() => setGameStatus("INIT")}>Play Again</button>
            </div>
          ) : (
            <div className="pot-buttons">
              <div className="pot-group">
                {" "}
                <p>Current Player: {currentPlayer}</p>{" "}
              </div>
              <div className="pot-group">
              {pots.map((pot, index) => (
    <button
        key={index}
        onClick={() => handlePlayerMove(index)}
        disabled={currentPlayer === 'COMP' || potsStatus[index] !== 'E'}
        className={`pot-button ${potsStatus[index].toLowerCase()}`}
        style={{ 
            borderRadius: '50%', 
            width: '50px', 
            height: '50px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontWeight: 'bold'
        }}
    >
        {pot}
    </button>
))}
              </div>
              <div className="pot-group">
                <p>Player Score: {userScore}</p>
              </div>
              <div className="pot-group">
                <p>Computer Score: {compScore}</p>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default App;

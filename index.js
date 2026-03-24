const express = require('express');
const cors = require('cors');

const app = express();
app.use(cors());
app.use(express.json());

const PORT = 3001;
const PROGRAM_ID = "54N5nsEJgPWf4ghPn6teZrseTxLo7wr1vLBMLGhruVgx";

app.get('/health', (req, res) => {
  res.json({ status: 'ok', program: PROGRAM_ID, network: 'devnet' });
});

app.post('/api/simulate-verify', (req, res) => {
  const { player1Summoner, player2Summoner } = req.body;
  res.json({
    matchId: 'DEMO-' + Date.now(),
    winner: Math.random() > 0.5 ? player1Summoner : player2Summoner,
    verified: true
  });
});

app.listen(PORT, () => console.log(` Backend en puerto ${PORT}`));
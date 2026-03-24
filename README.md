#  ChainArena — Esports Tournament Escrow on Solana

**ChainArena** es una plataforma descentralizada diseñada para gestionar torneos de eSports de forma transparente, segura y automatizada, eliminando la necesidad de intermediarios humanos para el reparto de premios.

##  Viabilidad Técnica e Integración
El proyecto utiliza la red **Solana Devnet** para garantizar transacciones de alta velocidad y bajo costo. La arquitectura se basa en un modelo de **Escrow (Fideicomiso)** donde los fondos de inscripción (Entry Fees) se bloquean en un Smart Contract hasta que un **Oracle externo** valida el resultado de la partida mediante la API oficial de Riot Games.

##  Arquitectura del Proyecto

El sistema se divide en tres componentes principales que interactúan entre sí:

1.  **Smart Contract (Anchor/Rust):**
    * Gestiona la creación de torneos con identificadores únicos.
    * Controla el registro de jugadores y el flujo de fondos (SOL) de forma segura.
    * Implementa lógica para que solo el Oracle autorizado pueda declarar al ganador.
    * Permite el reclamo de premios (Claim) únicamente al ganador verificado.

2.  **Backend Oracle (Node.js/Express):**
    * Actúa como puente entre el mundo Off-Chain (Datos de Riot Games) y la Blockchain.
    * Verifica los resultados de las partidas de forma automatizada.
    * Firma las transacciones de resolución de torneos para actualizar el estado en el contrato.

3.  **Frontend (Vanilla JS/Solana Web3.js):**
    * Interfaz reactiva con estética "Gamer".
    * Integración total con **Phantom Wallet** para firma de transacciones.
    * Gestión dinámica de estados para mostrar confirmaciones on-chain en tiempo real.

##  Flujo de Usuario (Protocolo)
1.  **Organizador:** Despliega el torneo en la blockchain pagando una pequeña renta de espacio.
2.  **Jugadores:** Se registran depositando el `Entry Fee` en SOL directamente al vault del contrato.
3.  **Resolución:** El Oracle consulta el resultado del juego. Si se valida, el Smart Contract asigna el `Winner`.
4.  **Premio:** El ganador reclama sus fondos directamente desde la dApp, sin esperar a un administrador.

##  Nivel de Complejidad y Seguridad
A diferencia de aplicaciones básicas, **ChainArena** implementa:
* **PDAs (Program Derived Addresses):** Para el almacenamiento seguro y organizado de los datos de cada torneo.
* **Validaciones On-Chain:** Comprobación estricta de firmas de Oracle y estados de torneo (Open/Finished) para evitar retiros no autorizados.
* **User Experience (UX):** Flujo de firma simplificado mediante la extensión de Phantom.

##  Estructura del Repositorio
* `/chain-arena-contract`: Código fuente en Rust (Anchor) desplegado en Devnet.
* `/chain-arena-backend`: Servidor Oracle en Node.js.
* `/chain-arena-frontend`: Cliente web para la interacción del usuario.

## 🔗 Detalles del Despliegue
* **Program ID:** `54N5nsEJgPWf4ghPn6teZrseTxLo7wr1vBMLGhruVgx`
* **Network:** Solana Devnet

--


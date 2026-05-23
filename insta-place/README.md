# Insta Place

Insta Place is a MERN-style project split into two apps:

- `Backend`: an Express API for users, places, image uploads, and authentication.
- `Frontend`: a React client that consumes the API.

## Project Structure

```text
insta-place/
├── Backend/
│   ├── app.js
│   ├── controllers/
│   ├── middleware/
│   ├── models/
│   └── routes/
└── Frontend/
    ├── public/
    └── src/
```

## Requirements

- Node.js and npm
- A MongoDB database

## Setup

### 1. Install backend dependencies

```bash
cd Backend
npm install
```

### 2. Install frontend dependencies

```bash
cd ../Frontend
npm install
```

### 3. Configure MongoDB

The backend currently connects directly in `Backend/app.js`. Replace `<password>` in the MongoDB connection string with your actual database password before starting the server.

## Run the App

### Start the backend

```bash
cd Backend
npm start
```

The backend listens on port `5000`.

### Start the frontend

```bash
cd Frontend
npm start
```

The frontend uses the React development server.

## Notes

- Uploaded images are served from `/uploads/images`.
- API routes are mounted under `/api/places` and `/api/users`.
- The backend uses `nodemon` through the `npm start` script.
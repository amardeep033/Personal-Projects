const jwt = require('jsonwebtoken');

const HttpError = require('../models/http-error');

module.exports = (req, res, next) => {
    if(req.method==='OPTIONS')
    {
    return next()
    }
  try 
  {
    const token = req.headers.authorization.split(' ')[1]; // Authorization: 'Bearer TOKEN'
    if (!token) 
    {
      throw new Error('Authentication failed!');
    }
    const decodedPayload = jwt.verify(token, 'thisissecretkey');
    req.userData = { userId: decodedPayload.userId };
    next();
  } 
  catch (err) 
  {
    return next(new HttpError(err.message,401))
  }
};

class HttpError extends Error{
    constructor(msg,ec)
    {
        super(msg)
        this.code=ec
    }
}

module.exports=HttpError
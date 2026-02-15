_ts_decorate([
    (0, _common.UseInterceptors)((0, _platformexpress.FileInterceptor)('file')),
    (0, _common.Post)('file'),
    _ts_param(0, (0, _common.Body)()),
    _ts_param(1, (0, _common.UploadedFile)(new _common.ParseFilePipeBuilder().addFileTypeValidator({
        fileType: "(png|jpg|gif|jpeg)"
    }).addMaxSizeValidator({
        maxSize: 5000000
    }).build({
        fileIsRequired: false
    }))),
    _ts_metadata("design:type", Function),
    _ts_metadata("design:paramtypes", [Object, Object]),
    _ts_metadata("design:returntype", Promise)
], AppController.prototype, "uploadFile", null);

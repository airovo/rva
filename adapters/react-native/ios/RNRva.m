#import <React/RCTBridgeModule.h>

@interface RCT_EXTERN_MODULE(RNRva, NSObject)

RCT_EXTERN_METHOD(resolve:(NSString *)uri
                  width:(nonnull NSNumber *)width
                  height:(nonnull NSNumber *)height
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

RCT_EXTERN_METHOD(renderPng:(NSString *)uri
                  width:(nonnull NSNumber *)width
                  height:(nonnull NSNumber *)height
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject)

@end

# \Api20100401StreamApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_stream**](Api20100401StreamApi.md#create_stream) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Streams.json | Create a Stream
[**update_stream**](Api20100401StreamApi.md#update_stream) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Streams/{Sid}.json | Stop a Stream using either the SID of the Stream resource or the `name` used when creating the resource



## create_stream

> models::ApiV2010AccountCallStream create_stream(account_sid, call_sid, url, name, track, status_callback, status_callback_method, parameter1_name, parameter1_value, parameter2_name, parameter2_value, parameter3_name, parameter3_value, parameter4_name, parameter4_value, parameter5_name, parameter5_value, parameter6_name, parameter6_value, parameter7_name, parameter7_value, parameter8_name, parameter8_value, parameter9_name, parameter9_value, parameter10_name, parameter10_value, parameter11_name, parameter11_value, parameter12_name, parameter12_value, parameter13_name, parameter13_value, parameter14_name, parameter14_value, parameter15_name, parameter15_value, parameter16_name, parameter16_value, parameter17_name, parameter17_value, parameter18_name, parameter18_value, parameter19_name, parameter19_value, parameter20_name, parameter20_value, parameter21_name, parameter21_value, parameter22_name, parameter22_value, parameter23_name, parameter23_value, parameter24_name, parameter24_value, parameter25_name, parameter25_value, parameter26_name, parameter26_value, parameter27_name, parameter27_value, parameter28_name, parameter28_value, parameter29_name, parameter29_value, parameter30_name, parameter30_value, parameter31_name, parameter31_value, parameter32_name, parameter32_value, parameter33_name, parameter33_value, parameter34_name, parameter34_value, parameter35_name, parameter35_value, parameter36_name, parameter36_value, parameter37_name, parameter37_value, parameter38_name, parameter38_value, parameter39_name, parameter39_value, parameter40_name, parameter40_value, parameter41_name, parameter41_value, parameter42_name, parameter42_value, parameter43_name, parameter43_value, parameter44_name, parameter44_value, parameter45_name, parameter45_value, parameter46_name, parameter46_value, parameter47_name, parameter47_value, parameter48_name, parameter48_value, parameter49_name, parameter49_value, parameter50_name, parameter50_value, parameter51_name, parameter51_value, parameter52_name, parameter52_value, parameter53_name, parameter53_value, parameter54_name, parameter54_value, parameter55_name, parameter55_value, parameter56_name, parameter56_value, parameter57_name, parameter57_value, parameter58_name, parameter58_value, parameter59_name, parameter59_value, parameter60_name, parameter60_value, parameter61_name, parameter61_value, parameter62_name, parameter62_value, parameter63_name, parameter63_value, parameter64_name, parameter64_value, parameter65_name, parameter65_value, parameter66_name, parameter66_value, parameter67_name, parameter67_value, parameter68_name, parameter68_value, parameter69_name, parameter69_value, parameter70_name, parameter70_value, parameter71_name, parameter71_value, parameter72_name, parameter72_value, parameter73_name, parameter73_value, parameter74_name, parameter74_value, parameter75_name, parameter75_value, parameter76_name, parameter76_value, parameter77_name, parameter77_value, parameter78_name, parameter78_value, parameter79_name, parameter79_value, parameter80_name, parameter80_value, parameter81_name, parameter81_value, parameter82_name, parameter82_value, parameter83_name, parameter83_value, parameter84_name, parameter84_value, parameter85_name, parameter85_value, parameter86_name, parameter86_value, parameter87_name, parameter87_value, parameter88_name, parameter88_value, parameter89_name, parameter89_value, parameter90_name, parameter90_value, parameter91_name, parameter91_value, parameter92_name, parameter92_value, parameter93_name, parameter93_value, parameter94_name, parameter94_value, parameter95_name, parameter95_value, parameter96_name, parameter96_value, parameter97_name, parameter97_value, parameter98_name, parameter98_value, parameter99_name, parameter99_value)
Create a Stream

Create a Stream

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Stream resource. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Stream resource is associated with. | [required] |
**url** | **String** | Relative or absolute URL where WebSocket connection will be established. | [required] |
**name** | Option<**String**> | The user-specified name of this Stream, if one was given when the Stream was created. This can be used to stop the Stream. |  |
**track** | Option<[**models::StreamEnumTrack**](StreamEnumTrack.md)> |  |  |
**status_callback** | Option<**String**> | Absolute URL to which Twilio sends status callback HTTP requests. |  |
**status_callback_method** | Option<**String**> | The HTTP method Twilio uses when sending `status_callback` requests. Possible values are `GET` and `POST`. Default is `POST`. |  |
**parameter1_name** | Option<**String**> | Parameter name |  |
**parameter1_value** | Option<**String**> | Parameter value |  |
**parameter2_name** | Option<**String**> | Parameter name |  |
**parameter2_value** | Option<**String**> | Parameter value |  |
**parameter3_name** | Option<**String**> | Parameter name |  |
**parameter3_value** | Option<**String**> | Parameter value |  |
**parameter4_name** | Option<**String**> | Parameter name |  |
**parameter4_value** | Option<**String**> | Parameter value |  |
**parameter5_name** | Option<**String**> | Parameter name |  |
**parameter5_value** | Option<**String**> | Parameter value |  |
**parameter6_name** | Option<**String**> | Parameter name |  |
**parameter6_value** | Option<**String**> | Parameter value |  |
**parameter7_name** | Option<**String**> | Parameter name |  |
**parameter7_value** | Option<**String**> | Parameter value |  |
**parameter8_name** | Option<**String**> | Parameter name |  |
**parameter8_value** | Option<**String**> | Parameter value |  |
**parameter9_name** | Option<**String**> | Parameter name |  |
**parameter9_value** | Option<**String**> | Parameter value |  |
**parameter10_name** | Option<**String**> | Parameter name |  |
**parameter10_value** | Option<**String**> | Parameter value |  |
**parameter11_name** | Option<**String**> | Parameter name |  |
**parameter11_value** | Option<**String**> | Parameter value |  |
**parameter12_name** | Option<**String**> | Parameter name |  |
**parameter12_value** | Option<**String**> | Parameter value |  |
**parameter13_name** | Option<**String**> | Parameter name |  |
**parameter13_value** | Option<**String**> | Parameter value |  |
**parameter14_name** | Option<**String**> | Parameter name |  |
**parameter14_value** | Option<**String**> | Parameter value |  |
**parameter15_name** | Option<**String**> | Parameter name |  |
**parameter15_value** | Option<**String**> | Parameter value |  |
**parameter16_name** | Option<**String**> | Parameter name |  |
**parameter16_value** | Option<**String**> | Parameter value |  |
**parameter17_name** | Option<**String**> | Parameter name |  |
**parameter17_value** | Option<**String**> | Parameter value |  |
**parameter18_name** | Option<**String**> | Parameter name |  |
**parameter18_value** | Option<**String**> | Parameter value |  |
**parameter19_name** | Option<**String**> | Parameter name |  |
**parameter19_value** | Option<**String**> | Parameter value |  |
**parameter20_name** | Option<**String**> | Parameter name |  |
**parameter20_value** | Option<**String**> | Parameter value |  |
**parameter21_name** | Option<**String**> | Parameter name |  |
**parameter21_value** | Option<**String**> | Parameter value |  |
**parameter22_name** | Option<**String**> | Parameter name |  |
**parameter22_value** | Option<**String**> | Parameter value |  |
**parameter23_name** | Option<**String**> | Parameter name |  |
**parameter23_value** | Option<**String**> | Parameter value |  |
**parameter24_name** | Option<**String**> | Parameter name |  |
**parameter24_value** | Option<**String**> | Parameter value |  |
**parameter25_name** | Option<**String**> | Parameter name |  |
**parameter25_value** | Option<**String**> | Parameter value |  |
**parameter26_name** | Option<**String**> | Parameter name |  |
**parameter26_value** | Option<**String**> | Parameter value |  |
**parameter27_name** | Option<**String**> | Parameter name |  |
**parameter27_value** | Option<**String**> | Parameter value |  |
**parameter28_name** | Option<**String**> | Parameter name |  |
**parameter28_value** | Option<**String**> | Parameter value |  |
**parameter29_name** | Option<**String**> | Parameter name |  |
**parameter29_value** | Option<**String**> | Parameter value |  |
**parameter30_name** | Option<**String**> | Parameter name |  |
**parameter30_value** | Option<**String**> | Parameter value |  |
**parameter31_name** | Option<**String**> | Parameter name |  |
**parameter31_value** | Option<**String**> | Parameter value |  |
**parameter32_name** | Option<**String**> | Parameter name |  |
**parameter32_value** | Option<**String**> | Parameter value |  |
**parameter33_name** | Option<**String**> | Parameter name |  |
**parameter33_value** | Option<**String**> | Parameter value |  |
**parameter34_name** | Option<**String**> | Parameter name |  |
**parameter34_value** | Option<**String**> | Parameter value |  |
**parameter35_name** | Option<**String**> | Parameter name |  |
**parameter35_value** | Option<**String**> | Parameter value |  |
**parameter36_name** | Option<**String**> | Parameter name |  |
**parameter36_value** | Option<**String**> | Parameter value |  |
**parameter37_name** | Option<**String**> | Parameter name |  |
**parameter37_value** | Option<**String**> | Parameter value |  |
**parameter38_name** | Option<**String**> | Parameter name |  |
**parameter38_value** | Option<**String**> | Parameter value |  |
**parameter39_name** | Option<**String**> | Parameter name |  |
**parameter39_value** | Option<**String**> | Parameter value |  |
**parameter40_name** | Option<**String**> | Parameter name |  |
**parameter40_value** | Option<**String**> | Parameter value |  |
**parameter41_name** | Option<**String**> | Parameter name |  |
**parameter41_value** | Option<**String**> | Parameter value |  |
**parameter42_name** | Option<**String**> | Parameter name |  |
**parameter42_value** | Option<**String**> | Parameter value |  |
**parameter43_name** | Option<**String**> | Parameter name |  |
**parameter43_value** | Option<**String**> | Parameter value |  |
**parameter44_name** | Option<**String**> | Parameter name |  |
**parameter44_value** | Option<**String**> | Parameter value |  |
**parameter45_name** | Option<**String**> | Parameter name |  |
**parameter45_value** | Option<**String**> | Parameter value |  |
**parameter46_name** | Option<**String**> | Parameter name |  |
**parameter46_value** | Option<**String**> | Parameter value |  |
**parameter47_name** | Option<**String**> | Parameter name |  |
**parameter47_value** | Option<**String**> | Parameter value |  |
**parameter48_name** | Option<**String**> | Parameter name |  |
**parameter48_value** | Option<**String**> | Parameter value |  |
**parameter49_name** | Option<**String**> | Parameter name |  |
**parameter49_value** | Option<**String**> | Parameter value |  |
**parameter50_name** | Option<**String**> | Parameter name |  |
**parameter50_value** | Option<**String**> | Parameter value |  |
**parameter51_name** | Option<**String**> | Parameter name |  |
**parameter51_value** | Option<**String**> | Parameter value |  |
**parameter52_name** | Option<**String**> | Parameter name |  |
**parameter52_value** | Option<**String**> | Parameter value |  |
**parameter53_name** | Option<**String**> | Parameter name |  |
**parameter53_value** | Option<**String**> | Parameter value |  |
**parameter54_name** | Option<**String**> | Parameter name |  |
**parameter54_value** | Option<**String**> | Parameter value |  |
**parameter55_name** | Option<**String**> | Parameter name |  |
**parameter55_value** | Option<**String**> | Parameter value |  |
**parameter56_name** | Option<**String**> | Parameter name |  |
**parameter56_value** | Option<**String**> | Parameter value |  |
**parameter57_name** | Option<**String**> | Parameter name |  |
**parameter57_value** | Option<**String**> | Parameter value |  |
**parameter58_name** | Option<**String**> | Parameter name |  |
**parameter58_value** | Option<**String**> | Parameter value |  |
**parameter59_name** | Option<**String**> | Parameter name |  |
**parameter59_value** | Option<**String**> | Parameter value |  |
**parameter60_name** | Option<**String**> | Parameter name |  |
**parameter60_value** | Option<**String**> | Parameter value |  |
**parameter61_name** | Option<**String**> | Parameter name |  |
**parameter61_value** | Option<**String**> | Parameter value |  |
**parameter62_name** | Option<**String**> | Parameter name |  |
**parameter62_value** | Option<**String**> | Parameter value |  |
**parameter63_name** | Option<**String**> | Parameter name |  |
**parameter63_value** | Option<**String**> | Parameter value |  |
**parameter64_name** | Option<**String**> | Parameter name |  |
**parameter64_value** | Option<**String**> | Parameter value |  |
**parameter65_name** | Option<**String**> | Parameter name |  |
**parameter65_value** | Option<**String**> | Parameter value |  |
**parameter66_name** | Option<**String**> | Parameter name |  |
**parameter66_value** | Option<**String**> | Parameter value |  |
**parameter67_name** | Option<**String**> | Parameter name |  |
**parameter67_value** | Option<**String**> | Parameter value |  |
**parameter68_name** | Option<**String**> | Parameter name |  |
**parameter68_value** | Option<**String**> | Parameter value |  |
**parameter69_name** | Option<**String**> | Parameter name |  |
**parameter69_value** | Option<**String**> | Parameter value |  |
**parameter70_name** | Option<**String**> | Parameter name |  |
**parameter70_value** | Option<**String**> | Parameter value |  |
**parameter71_name** | Option<**String**> | Parameter name |  |
**parameter71_value** | Option<**String**> | Parameter value |  |
**parameter72_name** | Option<**String**> | Parameter name |  |
**parameter72_value** | Option<**String**> | Parameter value |  |
**parameter73_name** | Option<**String**> | Parameter name |  |
**parameter73_value** | Option<**String**> | Parameter value |  |
**parameter74_name** | Option<**String**> | Parameter name |  |
**parameter74_value** | Option<**String**> | Parameter value |  |
**parameter75_name** | Option<**String**> | Parameter name |  |
**parameter75_value** | Option<**String**> | Parameter value |  |
**parameter76_name** | Option<**String**> | Parameter name |  |
**parameter76_value** | Option<**String**> | Parameter value |  |
**parameter77_name** | Option<**String**> | Parameter name |  |
**parameter77_value** | Option<**String**> | Parameter value |  |
**parameter78_name** | Option<**String**> | Parameter name |  |
**parameter78_value** | Option<**String**> | Parameter value |  |
**parameter79_name** | Option<**String**> | Parameter name |  |
**parameter79_value** | Option<**String**> | Parameter value |  |
**parameter80_name** | Option<**String**> | Parameter name |  |
**parameter80_value** | Option<**String**> | Parameter value |  |
**parameter81_name** | Option<**String**> | Parameter name |  |
**parameter81_value** | Option<**String**> | Parameter value |  |
**parameter82_name** | Option<**String**> | Parameter name |  |
**parameter82_value** | Option<**String**> | Parameter value |  |
**parameter83_name** | Option<**String**> | Parameter name |  |
**parameter83_value** | Option<**String**> | Parameter value |  |
**parameter84_name** | Option<**String**> | Parameter name |  |
**parameter84_value** | Option<**String**> | Parameter value |  |
**parameter85_name** | Option<**String**> | Parameter name |  |
**parameter85_value** | Option<**String**> | Parameter value |  |
**parameter86_name** | Option<**String**> | Parameter name |  |
**parameter86_value** | Option<**String**> | Parameter value |  |
**parameter87_name** | Option<**String**> | Parameter name |  |
**parameter87_value** | Option<**String**> | Parameter value |  |
**parameter88_name** | Option<**String**> | Parameter name |  |
**parameter88_value** | Option<**String**> | Parameter value |  |
**parameter89_name** | Option<**String**> | Parameter name |  |
**parameter89_value** | Option<**String**> | Parameter value |  |
**parameter90_name** | Option<**String**> | Parameter name |  |
**parameter90_value** | Option<**String**> | Parameter value |  |
**parameter91_name** | Option<**String**> | Parameter name |  |
**parameter91_value** | Option<**String**> | Parameter value |  |
**parameter92_name** | Option<**String**> | Parameter name |  |
**parameter92_value** | Option<**String**> | Parameter value |  |
**parameter93_name** | Option<**String**> | Parameter name |  |
**parameter93_value** | Option<**String**> | Parameter value |  |
**parameter94_name** | Option<**String**> | Parameter name |  |
**parameter94_value** | Option<**String**> | Parameter value |  |
**parameter95_name** | Option<**String**> | Parameter name |  |
**parameter95_value** | Option<**String**> | Parameter value |  |
**parameter96_name** | Option<**String**> | Parameter name |  |
**parameter96_value** | Option<**String**> | Parameter value |  |
**parameter97_name** | Option<**String**> | Parameter name |  |
**parameter97_value** | Option<**String**> | Parameter value |  |
**parameter98_name** | Option<**String**> | Parameter name |  |
**parameter98_value** | Option<**String**> | Parameter value |  |
**parameter99_name** | Option<**String**> | Parameter name |  |
**parameter99_value** | Option<**String**> | Parameter value |  |

### Return type

[**models::ApiV2010AccountCallStream**](api.v2010.account.call.stream.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_stream

> models::ApiV2010AccountCallStream update_stream(account_sid, call_sid, sid, status)
Stop a Stream using either the SID of the Stream resource or the `name` used when creating the resource

Stop a Stream using either the SID of the Stream resource or the `name` used when creating the resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Stream resource. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Stream resource is associated with. | [required] |
**sid** | **String** | The SID or the `name` of the Stream resource to be stopped | [required] |
**status** | [**models::StreamEnumUpdateStatus**](StreamEnumUpdateStatus.md) |  | [required] |

### Return type

[**models::ApiV2010AccountCallStream**](api.v2010.account.call.stream.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


# \Api20100401DomainApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_domain**](Api20100401DomainApi.md#create_sip_domain) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/Domains.json | Create a new Domain
[**delete_sip_domain**](Api20100401DomainApi.md#delete_sip_domain) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{Sid}.json | Delete an instance of a Domain
[**fetch_sip_domain**](Api20100401DomainApi.md#fetch_sip_domain) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{Sid}.json | Fetch an instance of a Domain
[**list_sip_domain**](Api20100401DomainApi.md#list_sip_domain) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains.json | Retrieve a list of domains belonging to the account used to make the request
[**update_sip_domain**](Api20100401DomainApi.md#update_sip_domain) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{Sid}.json | Update the attributes of a domain



## create_sip_domain

> models::ApiV2010AccountSipSipDomain create_sip_domain(account_sid, domain_name, friendly_name, voice_url, voice_method, voice_fallback_url, voice_fallback_method, voice_status_callback_url, voice_status_callback_method, sip_registration, emergency_calling_enabled, secure, byoc_trunk_sid, emergency_caller_sid)
Create a new Domain

Create a new Domain

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**domain_name** | **String** | The unique address you reserve on Twilio to which you route your SIP traffic. Domain names can contain letters, digits, and \\\"-\\\" and must end with `sip.twilio.com`. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe the resource. It can be up to 64 characters long. |  |
**voice_url** | Option<**String**> | The URL we should when the domain receives a call. |  |
**voice_method** | Option<**String**> | The HTTP method we should use to call `voice_url`. Can be: `GET` or `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while retrieving or executing the TwiML from `voice_url`. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method we should use to call `voice_fallback_url`. Can be: `GET` or `POST`. |  |
**voice_status_callback_url** | Option<**String**> | The URL that we should call to pass status parameters (such as call ended) to your application. |  |
**voice_status_callback_method** | Option<**String**> | The HTTP method we should use to call `voice_status_callback_url`. Can be: `GET` or `POST`. |  |
**sip_registration** | Option<**bool**> | Whether to allow SIP Endpoints to register with the domain to receive calls. Can be `true` or `false`. `true` allows SIP Endpoints to register with the domain to receive calls, `false` does not. |  |
**emergency_calling_enabled** | Option<**bool**> | Whether emergency calling is enabled for the domain. If enabled, allows emergency calls on the domain from phone numbers with validated addresses. |  |
**secure** | Option<**bool**> | Whether secure SIP is enabled for the domain. If enabled, TLS will be enforced and SRTP will be negotiated on all incoming calls to this sip domain. |  |
**byoc_trunk_sid** | Option<**String**> | The SID of the BYOC Trunk(Bring Your Own Carrier) resource that the Sip Domain will be associated with. |  |
**emergency_caller_sid** | Option<**String**> | Whether an emergency caller sid is configured for the domain. If present, this phone number will be used as the callback for the emergency call. |  |

### Return type

[**models::ApiV2010AccountSipSipDomain**](api.v2010.account.sip.sip_domain.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_domain

> delete_sip_domain(account_sid, sid)
Delete an instance of a Domain

Delete an instance of a Domain

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the SipDomain resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the SipDomain resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_domain

> models::ApiV2010AccountSipSipDomain fetch_sip_domain(account_sid, sid)
Fetch an instance of a Domain

Fetch an instance of a Domain

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the SipDomain resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the SipDomain resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomain**](api.v2010.account.sip.sip_domain.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_domain

> models::ListSipDomainResponse list_sip_domain(account_sid, page_size, page, page_token)
Retrieve a list of domains belonging to the account used to make the request

Retrieve a list of domains belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the SipDomain resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipDomainResponse**](ListSipDomainResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_sip_domain

> models::ApiV2010AccountSipSipDomain update_sip_domain(account_sid, sid, friendly_name, voice_fallback_method, voice_fallback_url, voice_method, voice_status_callback_method, voice_status_callback_url, voice_url, sip_registration, domain_name, emergency_calling_enabled, secure, byoc_trunk_sid, emergency_caller_sid)
Update the attributes of a domain

Update the attributes of a domain

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the SipDomain resource to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the SipDomain resource to update. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe the resource. It can be up to 64 characters long. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method we should use to call `voice_fallback_url`. Can be: `GET` or `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while retrieving or executing the TwiML requested by `voice_url`. |  |
**voice_method** | Option<**String**> | The HTTP method we should use to call `voice_url` |  |
**voice_status_callback_method** | Option<**String**> | The HTTP method we should use to call `voice_status_callback_url`. Can be: `GET` or `POST`. |  |
**voice_status_callback_url** | Option<**String**> | The URL that we should call to pass status parameters (such as call ended) to your application. |  |
**voice_url** | Option<**String**> | The URL we should call when the domain receives a call. |  |
**sip_registration** | Option<**bool**> | Whether to allow SIP Endpoints to register with the domain to receive calls. Can be `true` or `false`. `true` allows SIP Endpoints to register with the domain to receive calls, `false` does not. |  |
**domain_name** | Option<**String**> | The unique address you reserve on Twilio to which you route your SIP traffic. Domain names can contain letters, digits, and \\\"-\\\" and must end with `sip.twilio.com`. |  |
**emergency_calling_enabled** | Option<**bool**> | Whether emergency calling is enabled for the domain. If enabled, allows emergency calls on the domain from phone numbers with validated addresses. |  |
**secure** | Option<**bool**> | Whether secure SIP is enabled for the domain. If enabled, TLS will be enforced and SRTP will be negotiated on all incoming calls to this sip domain. |  |
**byoc_trunk_sid** | Option<**String**> | The SID of the BYOC Trunk(Bring Your Own Carrier) resource that the Sip Domain will be associated with. |  |
**emergency_caller_sid** | Option<**String**> | Whether an emergency caller sid is configured for the domain. If present, this phone number will be used as the callback for the emergency call. |  |

### Return type

[**models::ApiV2010AccountSipSipDomain**](api.v2010.account.sip.sip_domain.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


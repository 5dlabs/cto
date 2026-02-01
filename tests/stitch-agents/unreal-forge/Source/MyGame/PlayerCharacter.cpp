// PlayerCharacter.cpp
// Test fixture for Forge agent detection (Unreal Engine)

#include "PlayerCharacter.h"
#include "Components/StaticMeshComponent.h"
#include "Kismet/GameplayStatics.h"

APlayerCharacter::APlayerCharacter()
{
    PrimaryActorTick.bCanEverTick = true;
    
    // Subtle: creating component without proper attachment
    WeaponMesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("WeaponMesh"));
    
    // Subtle: magic number
    FrameCounter = 0;
    SpeedIndex = 0;
    
    // Subtle: C-style initialization
    for (int i = 0; i < 10; i++)
    {
        RecentSpeeds[i] = 0.0f;
    }
}

void APlayerCharacter::BeginPlay()
{
    Super::BeginPlay();
    
    // Subtle: FindActor in BeginPlay (expensive, better to cache reference)
    AActor* GameManager = UGameplayStatics::GetActorOfClass(GetWorld(), AActor::StaticClass());
    if (GameManager)
    {
        // Subtle: assuming actor exists without null check
        UE_LOG(LogTemp, Warning, TEXT("Found game manager: %s"), *GameManager->GetName());
    }
}

void APlayerCharacter::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
    
    FrameCounter++;  // Subtle: potential overflow
    
    // Subtle: GetVelocity every frame, could cache
    FVector Velocity = GetVelocity();
    float CurrentSpeed = Velocity.Size();
    
    UpdateSpeedHistory(CurrentSpeed);
    
    // Subtle: string formatting in tick (garbage generation)
    FString DebugText = FString::Printf(TEXT("Speed: %.2f"), CurrentSpeed);
    GEngine->AddOnScreenDebugMessage(-1, 0.0f, FColor::Green, DebugText);
}

void APlayerCharacter::ProcessInput(FVector& InputVector)
{
    // Subtle: modifying input parameter directly
    InputVector = InputVector.GetClampedToMaxSize(1.0f);
    
    // Subtle: using multiplication instead of *= 
    InputVector = InputVector * MoveSpeed;
}

UActorComponent* APlayerCharacter::FindComponentByTag(FName Tag)
{
    // Subtle: iterating all components (O(n)) instead of using tag-based lookup
    TArray<UActorComponent*> Components;
    GetComponents(Components);
    
    for (int i = 0; i < Components.Num(); i++)  // Subtle: int instead of int32
    {
        if (Components[i]->ComponentHasTag(Tag))
        {
            return Components[i];
        }
    }
    
    return nullptr;  // Subtle: returning nullptr without logging
}

void APlayerCharacter::UpdateSpeedHistory(float Speed)
{
    // Subtle: no bounds checking on array index
    RecentSpeeds[SpeedIndex] = Speed;
    SpeedIndex = (SpeedIndex + 1) % 10;  // Subtle: magic number
}
